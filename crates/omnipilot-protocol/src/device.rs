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

//! In-process contract implemented by every device adapter. The JSON-RPC
//! service is the out-of-process form of the same operations.

use async_trait::async_trait;
use omnipilot_core::{
    Action, ActionOutcome, Capability, DeviceInfo, LogEntry, Observation, OmniError,
};

#[async_trait]
pub trait DeviceAdapter: Send + Sync {
    /// Stable identity and current availability of the device.
    fn info(&self) -> DeviceInfo;

    /// Capabilities are queried after connection so the engine never assumes a
    /// platform supports an operation just because the action exists.
    async fn capabilities(&self) -> Vec<Capability>;

    async fn observe(&self, include_screenshot: bool) -> Result<Observation, OmniError>;

    async fn execute(&self, action: &Action) -> Result<ActionOutcome, OmniError>;

    async fn logs(&self, limit: usize) -> Result<Vec<LogEntry>, OmniError>;

    async fn shutdown(&self) -> Result<(), OmniError>;
}
