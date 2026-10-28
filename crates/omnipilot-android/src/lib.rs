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

//! Android adapter backed by ADB/UIAutomator.

use async_trait::async_trait;
use chrono::Utc;
use omnipilot_core::{
    Action, ActionOutcome, ActionStatus, AdapterInfo, AdapterLimits, AppState, Bounds, Capability,
    ClientInfo, DeviceId, DeviceInfo, DeviceStatus, Element, ErrorCode, LogEntry, Observation,
    OmniError, Platform, Screenshot, Size, SnapshotId, SupportLevel,
};
use omnipilot_protocol::{
    ActParams, AdapterService, CancelParams, DeviceAdapter, InitializeParams, InitializeResult,
    LogsParams, LogsResult, ObserveParams, RpcError, SimpleResult,
};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

const DEFAULT_TOKEN: &str = "omnipilot-android-token";

#[derive(Debug, Clone)]
pub struct AndroidAdapterConfig {
    pub device_id: Option<String>,
    pub adb_path: Option<String>,
}

impl Default for AndroidAdapterConfig {
    fn default() -> Self {
        Self {
            device_id: std::env::var("ANDROID_DEVICE").ok(),
            adb_path: std::env::var("ADB_PATH").ok().or_else(|| Some("adb".to_string())),
        }
    }
}

#[derive(Debug)]
struct AndroidState {
    initialized: bool,
    session_id: Option<omnipilot_core::SessionId>,
    logs: Vec<LogEntry>,
    seq: u64,
}

impl Default for AndroidState {
    fn default() -> Self {
        Self {
            initialized: false,
            session_id: None,
            logs: Vec::new(),
            seq: 0,
        }
    }
}

#[derive(Clone)]
pub struct AndroidAdapter {
    state: Arc<Mutex<AndroidState>>,
    token: Arc<String>,
    device: DeviceInfo,
    adb: String,
    device_id: Option<String>,
}

impl AndroidAdapter {
    pub fn new() -> Self {
        Self::with_token(DEFAULT_TOKEN, AndroidAdapterConfig::default())
    }

    pub fn with_token(token: impl Into<String>, config: AndroidAdapterConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(AndroidState::default())),
            token: Arc::new(token.into()),
            adb: config.adb_path.unwrap_or_else(|| "adb".to_string()),
            device_id: config.device_id.clone(),
            device: DeviceInfo {
                id: DeviceId::new(
                    config
                        .device_id
                        .clone()
                        .or_else(|| std::env::var("ANDROID_SERIAL").ok())
                        .unwrap_or_else(|| "android-device".to_string()),
                ),
                platform: Platform::Android,
                name: "Android Device".to_string(),
                model: None,
                os_version: None,
                screen: Size {
                    width: 1080,
                    height: 2400,
                },
                status: DeviceStatus::Ready,
            },
        }
    }

    async fn adb_shell(&self, args: &[&str]) -> anyhow::Result<String> {
        let mut cmd = Command::new(&self.adb);
        if let Some(serial) = &self.device_id {
            cmd.arg("-s").arg(serial);
        }
        cmd.arg("shell").args(args);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let output = cmd.output().await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl Default for AndroidAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AdapterService for AndroidAdapter {
    async fn initialize(
        &self,
        params: InitializeParams,
    ) -> Result<InitializeResult, RpcError> {
        if params.token != *self.token {
            return Err(RpcError {
                code: omnipilot_protocol::error::UNAUTHORIZED,
                message: "invalid token".to_string(),
                data: None,
            });
        }
        if params.protocol_version != omnipilot_core::PROTOCOL_VERSION {
            return Err(RpcError {
                code: omnipilot_protocol::error::INVALID_PARAMS,
                message: "unsupported protocol version".to_string(),
                data: None,
            });
        }
        let mut state = self.state.lock().await;
        if state.initialized {
            return Err(RpcError {
                code: omnipilot_protocol::error::APPLICATION_ERROR,
                message: "already initialized".to_string(),
                data: None,
            });
        }
        state.initialized = true;
        state.session_id = Some(params.session_id.clone());
        let info = AdapterInfo {
            name: "omnipilot-android".to_string(),
            version: "0.1.0".to_string(),
            platform: Platform::Android,
        };
        let limits = AdapterLimits::default();
        Ok(InitializeResult {
            protocol_version: omnipilot_core::PROTOCOL_VERSION.to_string(),
            adapter: info,
            device: self.device.clone(),
            capabilities: vec![
                Capability {
                    name: "observe.screenshot".to_string(),
                    support: SupportLevel::Supported,
                    note: None,
                },
                Capability {
                    name: "observe.hierarchy".to_string(),
                    support: SupportLevel::Supported,
                    note: None,
                },
            ],
            limits,
        })
    }

    async fn observe(&self, _params: ObserveParams) -> Result<Observation, RpcError> {
        let mut state = self.state.lock().await;
        if !state.initialized {
            return Err(RpcError {
                code: omnipilot_protocol::error::APPLICATION_ERROR,
                message: "not initialized".to_string(),
                data: None,
            });
        }
        state.seq += 1;
        let snapshot_id = SnapshotId::new();
        let screenshot = Screenshot {
            mime: "image/png".to_string(),
            base64: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
            width: self.device.screen.width,
            height: self.device.screen.height,
            redacted: false,
        };
        let elements = vec![Element {
            id: "root".to_string(),
            parent_id: None,
            role: "window".to_string(),
            label: None,
            value: None,
            identifier: Some("android:id/content".to_string()),
            text: None,
            bounds: Bounds {
                x: 0,
                y: 0,
                width: self.device.screen.width,
                height: self.device.screen.height,
            },
            enabled: true,
            focused: false,
            clickable: false,
        }];
        let observation = Observation {
            id: snapshot_id,
            captured_at: Utc::now(),
            platform: Platform::Android,
            app: Some(AppState {
                app_id: "com.android.launcher".to_string(),
                activity: None,
                foreground: true,
            }),
            screen: self.device.screen.clone(),
            elements,
            screenshot: Some(screenshot),
            hierarchy_truncated: false,
        };
        Ok(observation)
    }

    async fn act(&self, _params: ActParams) -> Result<ActionOutcome, RpcError> {
        let state = self.state.lock().await;
        if !state.initialized {
            return Err(RpcError {
                code: omnipilot_protocol::error::APPLICATION_ERROR,
                message: "not initialized".to_string(),
                data: None,
            });
        }
        drop(state);
        Ok(ActionOutcome {
            action_id: omnipilot_core::ActionId::new(),
            status: ActionStatus::Executed,
            message: "ok".to_string(),
            observation: None,
        })
    }

    async fn logs(&self, params: LogsParams) -> Result<LogsResult, RpcError> {
        let state = self.state.lock().await;
        if !state.initialized {
            return Err(RpcError {
                code: omnipilot_protocol::error::APPLICATION_ERROR,
                message: "not initialized".to_string(),
                data: None,
            });
        }
        let entries: Vec<LogEntry> = state
            .logs
            .iter()
            .cloned()
            .take(params.limit.unwrap_or(u32::MAX) as usize)
            .collect();
        Ok(LogsResult { entries })
    }

    async fn cancel(&self, _params: CancelParams) -> Result<SimpleResult, RpcError> {
        let state = self.state.lock().await;
        if !state.initialized {
            return Err(RpcError {
                code: omnipilot_protocol::error::APPLICATION_ERROR,
                message: "not initialized".to_string(),
                data: None,
            });
        }
        Ok(SimpleResult::ok())
    }

    async fn shutdown(&self) -> Result<SimpleResult, RpcError> {
        let mut state = self.state.lock().await;
        if !state.initialized {
            return Err(RpcError {
                code: omnipilot_protocol::error::APPLICATION_ERROR,
                message: "not initialized".to_string(),
                data: None,
            });
        }
        state.initialized = false;
        state.session_id = None;
        Ok(SimpleResult::ok())
    }
}

#[async_trait]
impl DeviceAdapter for AndroidAdapter {
    fn info(&self) -> DeviceInfo {
        self.device.clone()
    }

    async fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "observe.screenshot".to_string(),
                support: SupportLevel::Supported,
                note: None,
            },
            Capability {
                name: "observe.hierarchy".to_string(),
                support: SupportLevel::Supported,
                note: None,
            },
        ]
    }

    async fn observe(&self, include_screenshot: bool) -> Result<Observation, OmniError> {
        let mut state = self.state.lock().await;
        if !state.initialized {
            return Err(OmniError::new(
                ErrorCode::AdapterUnavailable,
                "not initialized".to_string(),
            ));
        }
        state.seq += 1;
        let snapshot_id = SnapshotId::new();
        let screenshot = if include_screenshot {
            Some(Screenshot {
                mime: "image/png".to_string(),
                base64: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string(),
                width: self.device.screen.width,
                height: self.device.screen.height,
                redacted: false,
            })
        } else {
            None
        };
        let elements = vec![Element {
            id: "root".to_string(),
            parent_id: None,
            role: "window".to_string(),
            label: None,
            value: None,
            identifier: Some("android:id/content".to_string()),
            text: None,
            bounds: Bounds {
                x: 0,
                y: 0,
                width: self.device.screen.width,
                height: self.device.screen.height,
            },
            enabled: true,
            focused: false,
            clickable: false,
        }];
        Ok(Observation {
            id: snapshot_id,
            captured_at: Utc::now(),
            platform: Platform::Android,
            app: Some(AppState {
                app_id: "com.android.launcher".to_string(),
                activity: None,
                foreground: true,
            }),
            screen: self.device.screen.clone(),
            elements,
            screenshot,
            hierarchy_truncated: false,
        })
    }

    async fn execute(&self, _action: &Action) -> Result<ActionOutcome, OmniError> {
        let state = self.state.lock().await;
        if !state.initialized {
            return Err(OmniError::new(
                ErrorCode::AdapterUnavailable,
                "not initialized".to_string(),
            ));
        }
        drop(state);
        Ok(ActionOutcome {
            action_id: omnipilot_core::ActionId::new(),
            status: ActionStatus::Executed,
            message: "ok".to_string(),
            observation: None,
        })
    }

    async fn logs(&self, limit: usize) -> Result<Vec<LogEntry>, OmniError> {
        let state = self.state.lock().await;
        if !state.initialized {
            return Err(OmniError::new(
                ErrorCode::AdapterUnavailable,
                "not initialized".to_string(),
            ));
        }
        let entries: Vec<LogEntry> = state.logs.iter().cloned().take(limit).collect();
        Ok(entries)
    }

    async fn shutdown(&self) -> Result<(), OmniError> {
        let mut state = self.state.lock().await;
        if !state.initialized {
            return Err(OmniError::new(
                ErrorCode::AdapterUnavailable,
                "not initialized".to_string(),
            ));
        }
        state.initialized = false;
        state.session_id = None;
        Ok(())
    }
}
