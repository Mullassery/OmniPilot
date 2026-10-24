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

//! Platform-neutral domain models shared by the OmniPilot engine, adapters,
//! protocol, CLI, and MCP server.
//!
//! The core deliberately knows nothing about a specific operating system. An
//! adapter reports observations in these types and executes these actions; the
//! engine owns task state, budgets, policy, verification, and evidence.

pub mod error;
pub mod models;

pub use error::{ErrorCode, OmniError};
pub use models::*;

/// Version of the adapter protocol implemented by this build.
pub const PROTOCOL_VERSION: &str = "omnipilot.adapter/1";
