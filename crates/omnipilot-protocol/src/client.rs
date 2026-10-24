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

//! Small sequential JSON-RPC client used by adapters and conformance tests.

use crate::error::ProtocolError;
use crate::rpc::{
    methods, ActParams, CancelParams, InitializeParams, InitializeResult, LogsParams, LogsResult,
    ObserveParams, RpcRequest, RpcResponse, SimpleResult,
};
use omnipilot_core::{Action, ActionOutcome, Observation, PROTOCOL_VERSION};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines};

pub struct ProtocolClient<R, W> {
    lines: Lines<BufReader<R>>,
    writer: W,
    next_id: u64,
}

impl<R, W> ProtocolClient<R, W>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            lines: BufReader::new(reader).lines(),
            writer,
            next_id: 1,
        }
    }

    pub async fn request<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<T, ProtocolError> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        let params = serde_json::to_value(params)?;
        let request = RpcRequest::new(id, method, Some(params));
        let encoded = serde_json::to_string(&request)?;
        self.writer.write_all(encoded.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;

        loop {
            let line = self.lines.next_line().await?.ok_or(ProtocolError::Closed)?;
            if line.trim().is_empty() {
                continue;
            }
            let response: RpcResponse = serde_json::from_str(&line)?;
            if response.id != id {
                return Err(ProtocolError::UnexpectedId {
                    expected: id,
                    actual: response.id,
                });
            }
            let value = response.into_result()?;
            return serde_json::from_value(value).map_err(ProtocolError::from);
        }
    }

    pub async fn initialize(
        &mut self,
        params: InitializeParams,
    ) -> Result<InitializeResult, ProtocolError> {
        let result: InitializeResult = self.request(methods::INITIALIZE, params).await?;
        if result.protocol_version != PROTOCOL_VERSION {
            return Err(ProtocolError::VersionMismatch {
                client: PROTOCOL_VERSION.to_string(),
                adapter: result.protocol_version,
            });
        }
        Ok(result)
    }

    pub async fn observe(&mut self, params: ObserveParams) -> Result<Observation, ProtocolError> {
        self.request(methods::OBSERVE, params).await
    }

    pub async fn act(&mut self, action: Action) -> Result<ActionOutcome, ProtocolError> {
        self.request(methods::ACT, ActParams::new(action)).await
    }

    pub async fn logs(&mut self, limit: Option<u32>) -> Result<LogsResult, ProtocolError> {
        self.request(methods::LOGS, LogsParams { limit }).await
    }

    pub async fn cancel(&mut self, reason: Option<String>) -> Result<SimpleResult, ProtocolError> {
        self.request(methods::CANCEL, CancelParams { reason }).await
    }

    pub async fn shutdown(&mut self) -> Result<SimpleResult, ProtocolError> {
        self.request(methods::SHUTDOWN, serde_json::json!({})).await
    }
}
