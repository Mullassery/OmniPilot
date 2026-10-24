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

//! Versioned, newline-delimited JSON-RPC 2.0 between the OmniPilot engine and a
//! device adapter. The transport is loopback-only in production; callers must
//! also supply the per-session bearer token in `initialize`.

pub mod adapter;
pub mod client;
pub mod device;
pub mod error;
pub mod rpc;
pub mod schema;

pub use adapter::{serve_ndjson, AdapterService};
pub use client::ProtocolClient;
pub use device::DeviceAdapter;
pub use error::{ProtocolError, RpcError};
pub use rpc::*;
pub use schema::protocol_schema;

pub use omnipilot_core::PROTOCOL_VERSION;
