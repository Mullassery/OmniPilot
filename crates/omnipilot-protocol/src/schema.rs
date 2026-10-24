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

//! Machine-readable protocol and task schemas, exposed by `omnipilot protocol
//! schema` and used to keep Kotlin/Swift adapters aligned with the Rust core.

use crate::rpc::{
    methods, ActParams, CancelParams, InitializeParams, InitializeResult, LogsParams, LogsResult,
    ObserveParams, SimpleResult,
};
use omnipilot_core::{
    Action, ActionOutcome, Assertion, Observation, TaskResult, TaskSpec, PROTOCOL_VERSION,
};
use schemars::schema_for;
use serde_json::{json, Value};

pub fn protocol_schema() -> Value {
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "jsonrpc": "2.0",
        "encoding": "newline-delimited JSON over loopback TCP",
        "handshake": {
            "method": methods::INITIALIZE,
            "token_required": true,
            "methods_before_initialize": "rejected"
        },
        "methods": methods::ALL,
        "schemas": {
            "InitializeParams": schema_for!(InitializeParams),
            "InitializeResult": schema_for!(InitializeResult),
            "ObserveParams": schema_for!(ObserveParams),
            "Observation": schema_for!(Observation),
            "ActParams": schema_for!(ActParams),
            "Action": schema_for!(Action),
            "ActionOutcome": schema_for!(ActionOutcome),
            "LogsParams": schema_for!(LogsParams),
            "LogsResult": schema_for!(LogsResult),
            "CancelParams": schema_for!(CancelParams),
            "SimpleResult": schema_for!(SimpleResult),
            "TaskSpec": schema_for!(TaskSpec),
            "TaskResult": schema_for!(TaskResult),
            "Assertion": schema_for!(Assertion),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_stable_and_contains_core_contract() {
        let schema = protocol_schema();
        assert_eq!(schema["protocol_version"], PROTOCOL_VERSION);
        assert_eq!(schema["jsonrpc"], "2.0");
        assert!(schema["schemas"]["Action"].is_object());
        assert!(schema["schemas"]["Observation"].is_object());
    }
}
