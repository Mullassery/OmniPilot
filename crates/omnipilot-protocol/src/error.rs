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

//! JSON-RPC and transport errors.

use omnipilot_core::OmniError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;

pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;
pub const APPLICATION_ERROR: i32 = -32000;
pub const UNAUTHORIZED: i32 = -32001;
pub const UNSUPPORTED: i32 = -32002;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(PARSE_ERROR, message)
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(INVALID_REQUEST, message)
    }

    pub fn method_not_found(method: &str) -> Self {
        Self::new(METHOD_NOT_FOUND, format!("method not found: {method}"))
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(INVALID_PARAMS, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(INTERNAL_ERROR, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(UNAUTHORIZED, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(UNSUPPORTED, message)
    }

    pub fn application(message: impl Into<String>) -> Self {
        Self::new(APPLICATION_ERROR, message)
    }

    pub fn from_omni(error: OmniError) -> Self {
        let code = match error.code {
            omnipilot_core::ErrorCode::UnsupportedAction => UNSUPPORTED,
            omnipilot_core::ErrorCode::PermissionDenied
            | omnipilot_core::ErrorCode::PolicyDenied => UNAUTHORIZED,
            _ => APPLICATION_ERROR,
        };
        Self {
            code,
            message: error.message.clone(),
            data: serde_json::to_value(error).ok(),
        }
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC {}: {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("transport I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Rpc(#[from] RpcError),
    #[error("adapter closed the connection")]
    Closed,
    #[error("protocol version mismatch: client={client} adapter={adapter}")]
    VersionMismatch { client: String, adapter: String },
    #[error("unexpected response id {actual}; expected {expected}")]
    UnexpectedId { expected: u64, actual: u64 },
    #[error("response contained neither result nor error")]
    EmptyResponse,
}

impl From<ProtocolError> for RpcError {
    fn from(value: ProtocolError) -> Self {
        match value {
            ProtocolError::Rpc(error) => error,
            ProtocolError::Json(error) => RpcError::parse(error.to_string()),
            other => RpcError::internal(other.to_string()),
        }
    }
}
