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

//! Normalized errors returned across the Rust core and adapter protocol.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// Stable error categories. Adapters map platform failures onto these codes so
/// the engine and MCP clients never have to parse free-form messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    UnsupportedAction,
    PermissionDenied,
    PolicyDenied,
    DeviceUnavailable,
    DeviceBusy,
    Timeout,
    Cancelled,
    StaleObservation,
    ElementNotFound,
    AppNotFound,
    AdapterUnavailable,
    VerificationFailed,
    EvidenceUnavailable,
    Internal,
}

impl ErrorCode {
    /// Whether retrying the same operation can reasonably succeed.
    pub fn default_retryable(self) -> bool {
        matches!(
            self,
            Self::DeviceBusy | Self::Timeout | Self::StaleObservation | Self::AdapterUnavailable
        )
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::UnsupportedAction => "unsupported_action",
            Self::PermissionDenied => "permission_denied",
            Self::PolicyDenied => "policy_denied",
            Self::DeviceUnavailable => "device_unavailable",
            Self::DeviceBusy => "device_busy",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
            Self::StaleObservation => "stale_observation",
            Self::ElementNotFound => "element_not_found",
            Self::AppNotFound => "app_not_found",
            Self::AdapterUnavailable => "adapter_unavailable",
            Self::VerificationFailed => "verification_failed",
            Self::EvidenceUnavailable => "evidence_unavailable",
            Self::Internal => "internal",
        }
    }
}

/// A serializable, platform-neutral error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OmniError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl OmniError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            retryable: code.default_retryable(),
            details: None,
        }
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidRequest, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::UnsupportedAction, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl fmt::Display for OmniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for OmniError {}

impl From<std::io::Error> for OmniError {
    fn from(value: std::io::Error) -> Self {
        Self::new(ErrorCode::Internal, value.to_string())
    }
}

impl From<serde_json::Error> for OmniError {
    fn from(value: serde_json::Error) -> Self {
        Self::new(ErrorCode::InvalidRequest, value.to_string())
    }
}
