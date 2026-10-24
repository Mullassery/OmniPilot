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

//! JSON-RPC envelopes and adapter method parameters.

use crate::error::RpcError;
use omnipilot_core::{Action, ActionId, LogEntry, Observation, SessionId, PROTOCOL_VERSION};
pub use omnipilot_core::{InitializeParams, InitializeResult};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSONRPC_VERSION: &str = "2.0";

pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const OBSERVE: &str = "observe";
    pub const ACT: &str = "act";
    pub const LOGS: &str = "logs";
    pub const CANCEL: &str = "cancel";
    pub const SHUTDOWN: &str = "shutdown";

    pub const ALL: &[&str] = &[INITIALIZE, OBSERVE, ACT, LOGS, CANCEL, SHUTDOWN];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl RpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }

    pub fn validate(&self) -> Result<(), RpcError> {
        if self.jsonrpc != JSONRPC_VERSION {
            return Err(RpcError::invalid_request(format!(
                "jsonrpc must be {JSONRPC_VERSION}"
            )));
        }
        if self.method.is_empty() {
            return Err(RpcError::invalid_request("method must not be empty"));
        }
        Ok(())
    }

    pub fn parse_params<T: DeserializeOwned>(&self) -> Result<T, RpcError> {
        let value = self
            .params
            .clone()
            .ok_or_else(|| RpcError::invalid_params("params are required"))?;
        serde_json::from_value(value)
            .map_err(|error| RpcError::invalid_params(format!("invalid params: {error}")))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn failure(id: u64, error: RpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }

    pub fn into_result(self) -> Result<Value, RpcError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.result
            .ok_or_else(|| RpcError::internal("empty JSON-RPC response"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObserveParams {
    #[serde(default = "default_true")]
    pub include_screenshot: bool,
    #[serde(default = "default_true")]
    pub include_hierarchy: bool,
}

impl Default for ObserveParams {
    fn default() -> Self {
        Self {
            include_screenshot: true,
            include_hierarchy: true,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<ActionId>,
    pub action: Action,
}

impl ActParams {
    pub fn new(action: Action) -> Self {
        Self {
            action_id: None,
            action,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LogsParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LogsResult {
    pub entries: Vec<LogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CancelParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SimpleResult {
    pub ok: bool,
    pub message: String,
}

impl SimpleResult {
    pub fn ok() -> Self {
        Self {
            ok: true,
            message: "ok".to_string(),
        }
    }
}

/// Validate the handshake fields shared by every adapter. Adapter-specific
/// token and session checks happen in the adapter's `initialize`.
pub fn validate_initialize(protocol_version: &str) -> Result<(), RpcError> {
    if protocol_version != PROTOCOL_VERSION {
        return Err(RpcError::invalid_params(format!(
            "unsupported protocol version {protocol_version}; expected {PROTOCOL_VERSION}"
        )));
    }
    Ok(())
}

/// Generate a fresh session id and a high-entropy bearer token. A real adapter
/// must compare the token in constant time and reject missing/incorrect tokens.
pub fn new_session() -> (SessionId, String) {
    (SessionId::new(), uuid::Uuid::new_v4().to_string())
}

/// Typed result for observation calls. Kept as a named alias so protocol users
/// do not depend on the JSON value shape.
pub type ObserveResult = Observation;
