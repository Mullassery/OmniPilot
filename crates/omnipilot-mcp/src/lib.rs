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

//! Model Context Protocol server exposing OmniPilot tasks, observations,
//! devices, and evidence. The server uses the official Rust SDK and stdio.

use anyhow::Context;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use omnipilot_core::{
    Action, Assertion, DeviceId, EvidenceId, Platform, RiskLevel, TaskId, TaskSpec,
    PROTOCOL_VERSION,
};
use omnipilot_engine::{Policy, Runtime, RuntimeConfig};
use omnipilot_ios::{IosAdapter, IosAdapterConfig};
use omnipilot_mock::MockAdapter;
use omnipilot_protocol::DeviceAdapter;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct OmniPilotServer {
    runtime: Arc<Runtime>,
    ios: Option<Arc<IosAdapter>>,
    tool_router: ToolRouter<Self>,
}

impl Default for OmniPilotServer {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniPilotServer {
    pub fn new() -> Self {
        Self::with_runtime(default_runtime())
    }

    pub fn with_runtime(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            ios: None,
            tool_router: Self::tool_router(),
        }
    }

    /// Build a server with the mock adapter plus a connected iOS adapter.
    /// The XCUITest process stays alive until the returned server is dropped
    /// or `run_stdio_with_ios` shuts it down.
    pub async fn with_ios(config: IosAdapterConfig) -> anyhow::Result<Self> {
        let ios = Arc::new(
            IosAdapter::connect(config)
                .await
                .context("failed to start the iOS adapter for MCP")?,
        );
        let adapters: Vec<Arc<dyn DeviceAdapter>> = vec![Arc::new(MockAdapter::new()), ios.clone()];
        Ok(Self {
            runtime: Arc::new(Runtime::new(adapters, runtime_config())),
            ios: Some(ios),
            tool_router: Self::tool_router(),
        })
    }

    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    fn resolve_platform(
        &self,
        requested: Option<Platform>,
        device_id: Option<&DeviceId>,
    ) -> Platform {
        requested
            .or_else(|| {
                device_id
                    .and_then(|id| self.runtime.device(id))
                    .map(|device| device.platform)
            })
            .unwrap_or(Platform::Mock)
    }
}

fn runtime_config() -> RuntimeConfig {
    let evidence_root = std::env::var("OMNIPILOT_TRACES")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("traces"));
    RuntimeConfig {
        evidence_root,
        policy: Policy::default(),
    }
}

fn default_runtime() -> Arc<Runtime> {
    Arc::new(Runtime::new(
        vec![Arc::new(MockAdapter::new())],
        runtime_config(),
    ))
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeviceArgs {
    /// Device id from `list_devices`.
    pub device_id: DeviceId,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ObserveArgs {
    pub device_id: DeviceId,
    /// Include the screenshot payload in the observation. Defaults to false.
    #[serde(default)]
    pub include_screenshot: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LogsArgs {
    pub device_id: DeviceId,
    #[serde(default = "default_log_limit")]
    pub limit: usize,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RunTaskArgs {
    /// What the user wants to achieve, in natural language.
    pub goal: String,
    /// Platform to run on. Defaults to the mock device.
    #[serde(default)]
    pub platform: Option<Platform>,
    #[serde(default)]
    pub device_id: Option<DeviceId>,
    #[serde(default)]
    pub app_id: Option<String>,
    /// Explicit, typed plan. Natural-language planning is not enabled in this build.
    #[serde(default)]
    pub steps: Vec<Action>,
    /// Assertions evaluated against a fresh final observation.
    #[serde(default)]
    pub success_criteria: Vec<Assertion>,
    #[serde(default = "default_max_steps")]
    pub max_steps: u32,
    #[serde(default = "default_max_duration_ms")]
    pub max_duration_ms: u64,
    /// Approve UI mutations if local policy requires approval.
    #[serde(default)]
    pub approve_ui: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskArgs {
    pub task_id: TaskId,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EvidenceArgs {
    pub task_id: TaskId,
    pub evidence_id: EvidenceId,
    /// Include the raw bytes as base64. Defaults to false.
    #[serde(default)]
    pub include_content: bool,
}

fn default_log_limit() -> usize {
    100
}

fn default_max_steps() -> u32 {
    20
}

fn default_max_duration_ms() -> u64 {
    120_000
}

#[derive(Debug, Serialize)]
struct DeviceList {
    protocol_version: &'static str,
    devices: Vec<omnipilot_core::DeviceInfo>,
}

#[tool_router]
impl OmniPilotServer {
    #[tool(description = "List devices available to this OmniPilot process.")]
    async fn list_devices(&self) -> Result<CallToolResult, McpError> {
        json_result(&DeviceList {
            protocol_version: PROTOCOL_VERSION,
            devices: self.runtime.devices(),
        })
    }

    #[tool(description = "Get the honest capability matrix for a device.")]
    async fn get_capabilities(
        &self,
        Parameters(args): Parameters<DeviceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let capabilities = self
            .runtime
            .capabilities(&args.device_id)
            .await
            .map_err(to_mcp_error)?;
        json_result(&json!({
            "device_id": args.device_id,
            "capabilities": capabilities,
        }))
    }

    #[tool(description = "Observe the current screen and accessibility hierarchy.")]
    async fn observe(
        &self,
        Parameters(args): Parameters<ObserveArgs>,
    ) -> Result<CallToolResult, McpError> {
        let observation = self
            .runtime
            .observe(&args.device_id, args.include_screenshot)
            .await
            .map_err(to_mcp_error)?;
        json_result(&observation)
    }

    #[tool(description = "Read recent device logs.")]
    async fn get_logs(
        &self,
        Parameters(args): Parameters<LogsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let entries = self
            .runtime
            .logs(&args.device_id, args.limit.min(500))
            .await
            .map_err(to_mcp_error)?;
        json_result(&json!({ "entries": entries }))
    }

    #[tool(
        description = "Start an explicit task plan. Returns a task id immediately; poll get_task for the verified result."
    )]
    async fn run_task(
        &self,
        Parameters(args): Parameters<RunTaskArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.steps.is_empty() && args.success_criteria.is_empty() {
            return Err(McpError::invalid_params(
                "supply at least one typed step or success criterion; natural-language planning is not enabled in this build",
                None,
            ));
        }

        let platform = self.resolve_platform(args.platform, args.device_id.as_ref());
        let mut spec = TaskSpec::new(args.goal, platform);
        spec.device_id = args.device_id;
        spec.app_id = args.app_id;
        spec.steps = args.steps;
        spec.success_criteria = args.success_criteria;
        spec.budgets.max_steps = args.max_steps;
        spec.budgets.max_duration_ms = args.max_duration_ms;
        if args.approve_ui {
            spec.approved_risks.push(RiskLevel::UiMutation);
        }

        let task_id = self.runtime.spawn_task(spec).map_err(to_mcp_error)?;
        json_result(&json!({
            "task_id": task_id,
            "status": "queued",
            "next": "call get_task with this task_id",
        }))
    }

    #[tool(description = "Get a task record and its verified result, assertions, and evidence.")]
    async fn get_task(
        &self,
        Parameters(args): Parameters<TaskArgs>,
    ) -> Result<CallToolResult, McpError> {
        let record = self.runtime.task(&args.task_id).ok_or_else(|| {
            McpError::invalid_params(format!("task {} was not found", args.task_id), None)
        })?;
        json_result(&record)
    }

    #[tool(description = "Cancel a queued or running task.")]
    async fn cancel_task(
        &self,
        Parameters(args): Parameters<TaskArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cancelled = self.runtime.cancel_task(&args.task_id);
        json_result(&json!({
            "task_id": args.task_id,
            "cancelled": cancelled,
        }))
    }

    #[tool(description = "Read evidence metadata and optionally its raw base64 content.")]
    async fn get_evidence(
        &self,
        Parameters(args): Parameters<EvidenceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let (reference, bytes) = self
            .runtime
            .evidence(&args.task_id, &args.evidence_id)
            .map_err(to_mcp_error)?;
        let content = args.include_content.then(|| STANDARD.encode(&bytes));
        json_result(&json!({
            "evidence": reference,
            "content_base64": content,
        }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OmniPilotServer {}

fn json_result<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| McpError::internal_error(error.to_string(), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

fn to_mcp_error(error: omnipilot_core::OmniError) -> McpError {
    let data = serde_json::to_value(&error).ok();
    McpError::new(rmcp::model::ErrorCode::INVALID_PARAMS, error.message, data)
}

pub async fn run_stdio() -> anyhow::Result<()> {
    run_stdio_with_ios(None).await
}

/// Serve MCP over stdio. When `ios` is supplied, the XCUITest adapter is
/// launched first and both the mock and iOS devices are exposed.
pub async fn run_stdio_with_ios(ios: Option<IosAdapterConfig>) -> anyhow::Result<()> {
    let server = match ios {
        Some(config) => OmniPilotServer::with_ios(config).await?,
        None => OmniPilotServer::new(),
    };
    let ios_adapter = server.ios.clone();
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .context("failed to start OmniPilot MCP server")?;
    let result = service.waiting().await.context("MCP server exited");
    if let Some(adapter) = ios_adapter {
        let _ = adapter.shutdown().await;
    }
    result?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_is_inferred_from_device_when_omitted() {
        let server = OmniPilotServer::new();
        let mock = DeviceId::new("mock-device-1");
        assert_eq!(server.resolve_platform(None, Some(&mock)), Platform::Mock);
        assert_eq!(
            server.resolve_platform(Some(Platform::Ios), Some(&mock)),
            Platform::Ios
        );
        assert_eq!(server.resolve_platform(None, None), Platform::Mock);
    }
}
