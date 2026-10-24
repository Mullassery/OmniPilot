// Copyright 2026 OmniPilot contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! iOS adapter backed by a long-lived XCUITest process.
//!
//! The host launches `xcodebuild` with the `OmniPilotAdapter` scheme. The UI
//! test prints a loopback port and a per-session token, then serves the normal
//! `omnipilot.adapter/1` NDJSON protocol. This crate owns that process and
//! exposes it through the same in-process `DeviceAdapter` contract as the mock.

use async_trait::async_trait;
use omnipilot_core::{
    Action, ActionOutcome, AdapterLimits, Capability, ClientInfo, DeviceInfo, ErrorCode, LogEntry,
    Observation, OmniError, SessionId, PROTOCOL_VERSION,
};
use omnipilot_protocol::error::{
    APPLICATION_ERROR, INVALID_PARAMS, METHOD_NOT_FOUND, UNAUTHORIZED, UNSUPPORTED,
};
use omnipilot_protocol::{
    InitializeParams, ObserveParams, ProtocolClient, ProtocolError, RpcError,
};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

const READY_PREFIX: &str = "OMNIPILOT_ADAPTER_READY";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const INITIALIZE_TIMEOUT: Duration = Duration::from_secs(15);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

/// Everything needed to start the XCUITest adapter. Paths are interpreted
/// relative to the process working directory, which should normally be the
/// OmniPilot repository root.
#[derive(Debug, Clone)]
pub struct IosAdapterConfig {
    pub project: PathBuf,
    pub scheme: String,
    pub destination: String,
    pub test_target: String,
    pub developer_dir: Option<PathBuf>,
    pub launch_timeout: Duration,
}

impl Default for IosAdapterConfig {
    fn default() -> Self {
        Self {
            project: PathBuf::from("apple/OmniPilot.xcodeproj"),
            scheme: "OmniPilotAdapter".to_string(),
            destination: "platform=iOS Simulator,name=iPhone 17".to_string(),
            test_target: "OmniPilotAdapterTests/OmniPilotAdapterTests/testServeAdapter".to_string(),
            developer_dir: std::env::var_os("DEVELOPER_DIR")
                .map(PathBuf::from)
                .or_else(|| {
                    let path = PathBuf::from("/Applications/Xcode.app/Contents/Developer");
                    path.is_dir().then_some(path)
                }),
            launch_timeout: Duration::from_secs(180),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IosError {
    #[error("iOS adapter I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("iOS adapter protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("could not start xcodebuild: {0}")]
    Launch(String),
    #[error("timed out after {0:?}")]
    Timeout(Duration),
    #[error("xcodebuild exited before the adapter became ready: {0}")]
    Exited(String),
    #[error("invalid adapter ready line: {0}")]
    InvalidReady(String),
    #[error("could not connect to the adapter: {0}")]
    Connect(String),
}

impl From<IosError> for OmniError {
    fn from(error: IosError) -> Self {
        match error {
            IosError::Protocol(error) => protocol_error_to_omni(error),
            IosError::Timeout(duration) => OmniError::new(
                ErrorCode::Timeout,
                format!("iOS adapter timed out after {duration:?}"),
            )
            .with_retryable(true),
            IosError::Io(error) => OmniError::new(
                ErrorCode::AdapterUnavailable,
                format!("iOS adapter I/O error: {error}"),
            )
            .with_retryable(true),
            other => OmniError::new(ErrorCode::AdapterUnavailable, other.to_string())
                .with_retryable(true),
        }
    }
}

/// A connected, initialized iOS adapter. The XCUITest process is kept alive
/// until `shutdown` or until this value is dropped.
pub struct IosAdapter {
    info: DeviceInfo,
    capabilities: Vec<Capability>,
    limits: AdapterLimits,
    client: Mutex<ProtocolClient<OwnedReadHalf, OwnedWriteHalf>>,
    child: Mutex<Option<Child>>,
}

impl IosAdapter {
    /// Launch XCUITest, wait for its loopback announcement, connect, and
    /// complete the protocol handshake.
    pub async fn connect(config: IosAdapterConfig) -> Result<Self, IosError> {
        let mut command = Command::new("xcodebuild");
        command
            .arg("-project")
            .arg(&config.project)
            .arg("-scheme")
            .arg(&config.scheme)
            .arg("-destination")
            .arg(&config.destination)
            .arg(format!("-only-testing:{}", config.test_target))
            .arg("test")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(developer_dir) = &config.developer_dir {
            command.env("DEVELOPER_DIR", developer_dir);
        }

        let mut child = command.spawn().map_err(|error| {
            IosError::Launch(format!("{} (project={})", error, config.project.display()))
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| IosError::Launch("xcodebuild stdout was not captured".to_string()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| IosError::Launch("xcodebuild stderr was not captured".to_string()))?;

        let stderr_tail = Arc::new(StdMutex::new(VecDeque::new()));
        let _stderr_task = tokio::spawn(drain_stderr(stderr, Arc::clone(&stderr_tail)));

        let mut lines = BufReader::new(stdout).lines();
        let announcement = timeout(config.launch_timeout, async {
            loop {
                match lines.next_line().await {
                    Ok(Some(line)) => {
                        if let Some(ready) = parse_ready_line(&line) {
                            return Ok(ready);
                        }
                        if !line.contains(READY_PREFIX) {
                            tracing::debug!(target: "omnipilot_ios", "xcodebuild: {line}");
                        }
                    }
                    Ok(None) => {
                        return Err(IosError::Exited(stderr_snapshot(&stderr_tail)));
                    }
                    Err(error) => return Err(IosError::Io(error)),
                }
            }
        })
        .await;

        let (port, token) = match announcement {
            Ok(Ok(ready)) => ready,
            Ok(Err(error)) => {
                stop_child(&mut child).await;
                return Err(error);
            }
            Err(_) => {
                stop_child(&mut child).await;
                return Err(IosError::Timeout(config.launch_timeout));
            }
        };

        let _stdout_task = tokio::spawn(drain_stdout(lines));

        let stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(("127.0.0.1", port)))
            .await
            .map_err(|_| IosError::Timeout(CONNECT_TIMEOUT))?
            .map_err(|error| IosError::Connect(error.to_string()))?;
        stream.set_nodelay(true)?;

        let (read_half, write_half) = stream.into_split();
        let mut client = ProtocolClient::new(read_half, write_half);
        let params = InitializeParams {
            protocol_version: PROTOCOL_VERSION.to_string(),
            session_id: SessionId::new(),
            token,
            client: ClientInfo {
                name: "omnipilot".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };
        let initialized = timeout(INITIALIZE_TIMEOUT, client.initialize(params))
            .await
            .map_err(|_| IosError::Timeout(INITIALIZE_TIMEOUT))?
            .map_err(IosError::Protocol)?;

        tracing::debug!(
            target: "omnipilot_ios",
            device = %initialized.device.id,
            "iOS adapter initialized"
        );

        Ok(Self {
            info: initialized.device,
            capabilities: initialized.capabilities,
            limits: initialized.limits,
            client: Mutex::new(client),
            child: Mutex::new(Some(child)),
        })
    }

    pub fn limits(&self) -> &AdapterLimits {
        &self.limits
    }

    pub fn capabilities_snapshot(&self) -> &[Capability] {
        &self.capabilities
    }

    /// Send `shutdown`, then wait briefly for xcodebuild to exit before
    /// killing it. Cleanup is best-effort so a broken adapter cannot hang the
    /// host.
    pub async fn shutdown(&self) -> Result<(), IosError> {
        {
            let mut client = self.client.lock().await;
            let _ = timeout(SHUTDOWN_TIMEOUT, client.shutdown()).await;
        }

        let mut child = self.child.lock().await;
        if let Some(mut child) = child.take() {
            match timeout(SHUTDOWN_TIMEOUT, child.wait()).await {
                Ok(Ok(_)) => {}
                _ => stop_child(&mut child).await,
            }
        }
        Ok(())
    }
}

#[async_trait]
impl omnipilot_protocol::DeviceAdapter for IosAdapter {
    fn info(&self) -> DeviceInfo {
        self.info.clone()
    }

    async fn capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    async fn observe(&self, include_screenshot: bool) -> Result<Observation, OmniError> {
        let mut client = self.client.lock().await;
        client
            .observe(ObserveParams {
                include_screenshot,
                include_hierarchy: true,
            })
            .await
            .map_err(|error| IosError::Protocol(error).into())
    }

    async fn execute(&self, action: &Action) -> Result<ActionOutcome, OmniError> {
        action.validate()?;
        let mut client = self.client.lock().await;
        client
            .act(action.clone())
            .await
            .map_err(|error| IosError::Protocol(error).into())
    }

    async fn logs(&self, limit: usize) -> Result<Vec<LogEntry>, OmniError> {
        let limit = u32::try_from(limit).unwrap_or(u32::MAX);
        let mut client = self.client.lock().await;
        client
            .logs(Some(limit))
            .await
            .map(|result| result.entries)
            .map_err(|error| IosError::Protocol(error).into())
    }

    async fn shutdown(&self) -> Result<(), OmniError> {
        IosAdapter::shutdown(self).await.map_err(OmniError::from)
    }
}

fn parse_ready_line(line: &str) -> Option<(u16, String)> {
    let rest = line.trim().strip_prefix(READY_PREFIX)?;
    let mut port = None;
    let mut token = None;
    for part in rest.split_whitespace() {
        if let Some(value) = part.strip_prefix("port=") {
            port = value.parse::<u16>().ok();
        } else if let Some(value) = part.strip_prefix("token=") {
            if !value.is_empty() {
                token = Some(value.to_string());
            }
        }
    }
    Some((port?, token?))
}

fn loggable(line: &str) -> &str {
    if line.contains(READY_PREFIX) {
        "OMNIPILOT_ADAPTER_READY <redacted>"
    } else {
        line
    }
}

async fn drain_stdout(mut lines: Lines<BufReader<ChildStdout>>) {
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::debug!(target: "omnipilot_ios", "xcodebuild: {}", loggable(&line));
    }
}

async fn drain_stderr(stderr: ChildStderr, tail: Arc<StdMutex<VecDeque<String>>>) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::debug!(target: "omnipilot_ios", "xcodebuild stderr: {line}");
        if let Ok(mut tail) = tail.lock() {
            tail.push_back(line);
            while tail.len() > 30 {
                tail.pop_front();
            }
        }
    }
}

fn stderr_snapshot(tail: &Arc<StdMutex<VecDeque<String>>>) -> String {
    let lines = tail
        .lock()
        .map(|tail| tail.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    if lines.is_empty() {
        "no stderr output".to_string()
    } else {
        lines.join("\n")
    }
}

async fn stop_child(child: &mut Child) {
    let _ = child.start_kill();
    let _ = child.wait().await;
}

fn protocol_error_to_omni(error: ProtocolError) -> OmniError {
    match error {
        ProtocolError::Rpc(rpc) => rpc_to_omni(rpc),
        ProtocolError::Closed => OmniError::new(
            ErrorCode::AdapterUnavailable,
            "iOS adapter closed the connection",
        )
        .with_retryable(true),
        ProtocolError::Io(error) => OmniError::new(
            ErrorCode::AdapterUnavailable,
            format!("iOS adapter I/O error: {error}"),
        )
        .with_retryable(true),
        ProtocolError::Json(error) => OmniError::new(
            ErrorCode::InvalidRequest,
            format!("invalid iOS adapter JSON: {error}"),
        ),
        other => {
            OmniError::new(ErrorCode::AdapterUnavailable, other.to_string()).with_retryable(true)
        }
    }
}

fn rpc_to_omni(error: RpcError) -> OmniError {
    let kind = error
        .data
        .as_ref()
        .and_then(|data| data.get("kind"))
        .and_then(|kind| kind.as_str());
    let code = match kind {
        Some("element_not_found") => ErrorCode::ElementNotFound,
        Some("app_not_found") => ErrorCode::AppNotFound,
        Some("cancelled") => ErrorCode::Cancelled,
        Some("timeout") => ErrorCode::Timeout,
        Some("unsupported") => ErrorCode::UnsupportedAction,
        Some("device_unavailable") => ErrorCode::DeviceUnavailable,
        Some("adapter_unavailable") => ErrorCode::AdapterUnavailable,
        Some("policy_denied") => ErrorCode::PolicyDenied,
        Some("permission_denied") => ErrorCode::PermissionDenied,
        _ => match error.code {
            UNSUPPORTED => ErrorCode::UnsupportedAction,
            UNAUTHORIZED => ErrorCode::PermissionDenied,
            INVALID_PARAMS => ErrorCode::InvalidRequest,
            METHOD_NOT_FOUND => ErrorCode::UnsupportedAction,
            APPLICATION_ERROR => ErrorCode::AdapterUnavailable,
            _ => ErrorCode::Internal,
        },
    };
    let retryable = error
        .data
        .as_ref()
        .and_then(|data| data.get("retryable"))
        .and_then(|retryable| retryable.as_bool())
        .unwrap_or_else(|| code.default_retryable());
    let mut omni = OmniError::new(code, error.message).with_retryable(retryable);
    if let Some(data) = error.data {
        omni = omni.with_details(data);
    }
    omni
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ready_announcement() {
        let parsed = parse_ready_line(
            "OMNIPILOT_ADAPTER_READY port=57274 token=8A6D3720-54BE-42F9-BFB4-D38EAC7123B6",
        );
        assert_eq!(
            parsed,
            Some((57274, "8A6D3720-54BE-42F9-BFB4-D38EAC7123B6".to_string()))
        );
    }

    #[test]
    fn rejects_incomplete_ready_announcement() {
        assert_eq!(parse_ready_line("OMNIPILOT_ADAPTER_READY port=57274"), None);
        assert_eq!(
            parse_ready_line("OMNIPILOT_ADAPTER_READY token=secret"),
            None
        );
        assert_eq!(parse_ready_line("something else"), None);
    }

    #[test]
    fn redacts_token_from_ready_line() {
        assert_eq!(
            loggable("OMNIPILOT_ADAPTER_READY port=1 token=secret"),
            "OMNIPILOT_ADAPTER_READY <redacted>"
        );
        assert_eq!(loggable("ordinary log"), "ordinary log");
    }
}
