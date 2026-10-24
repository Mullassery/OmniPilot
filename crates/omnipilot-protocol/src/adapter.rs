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

//! Adapter service interface and newline-delimited JSON-RPC server.

use crate::error::{ProtocolError, RpcError};
use crate::rpc::{
    methods, ActParams, CancelParams, InitializeParams, InitializeResult, LogsParams, LogsResult,
    ObserveParams, RpcRequest, RpcResponse, SimpleResult,
};
use async_trait::async_trait;
use omnipilot_core::{Action, ActionOutcome, Observation};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

/// The adapter-side implementation of the device protocol. Implementations
/// must authenticate the `initialize` token and report unsupported capabilities
/// honestly rather than silently degrading.
#[async_trait]
pub trait AdapterService: Send + Sync {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult, RpcError>;
    async fn observe(&self, params: ObserveParams) -> Result<Observation, RpcError>;
    async fn act(&self, params: ActParams) -> Result<ActionOutcome, RpcError>;
    async fn logs(&self, params: LogsParams) -> Result<LogsResult, RpcError>;
    async fn cancel(&self, params: CancelParams) -> Result<SimpleResult, RpcError>;
    async fn shutdown(&self) -> Result<SimpleResult, RpcError>;
}

/// Serve one adapter session over newline-delimited JSON-RPC.
///
/// The server enforces handshake-before-use. It does not add a second thread or
/// background task: the caller owns the transport lifetime.
pub async fn serve_ndjson<S, R, W>(
    reader: R,
    mut writer: W,
    service: S,
) -> Result<(), ProtocolError>
where
    S: AdapterService,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    let mut initialized = false;

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: RpcRequest = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(error) => {
                write_response(
                    &mut writer,
                    RpcResponse::failure(0, RpcError::parse(error.to_string())),
                )
                .await?;
                continue;
            }
        };

        let request_id = request.id;
        let method = request.method.clone();

        let result = if let Err(error) = request.validate() {
            Err(error)
        } else if method != methods::INITIALIZE && !initialized {
            Err(RpcError::unauthorized(
                "initialize must succeed before other methods",
            ))
        } else if method == methods::INITIALIZE && initialized {
            Err(RpcError::invalid_request("session is already initialized"))
        } else {
            dispatch(&service, &request).await
        };

        if method == methods::INITIALIZE && result.is_ok() {
            initialized = true;
        }

        let response = match result {
            Ok(value) => RpcResponse::success(request_id, value),
            Err(error) => RpcResponse::failure(request_id, error),
        };
        write_response(&mut writer, response).await?;

        if method == methods::SHUTDOWN {
            break;
        }
    }

    Ok(())
}

async fn dispatch<S: AdapterService>(service: &S, request: &RpcRequest) -> Result<Value, RpcError> {
    match request.method.as_str() {
        methods::INITIALIZE => {
            let params: InitializeParams = request.parse_params()?;
            let result: InitializeResult = service.initialize(params).await?;
            to_value(result)
        }
        methods::OBSERVE => {
            let params: ObserveParams = request.parse_params()?;
            let result: Observation = service.observe(params).await?;
            to_value(result)
        }
        methods::ACT => {
            let params: ActParams = request.parse_params()?;
            params
                .action
                .validate()
                .map_err(|error| RpcError::invalid_params(error.message))?;
            let result: ActionOutcome = service.act(params).await?;
            to_value(result)
        }
        methods::LOGS => {
            let params: LogsParams = request.parse_params()?;
            let result: LogsResult = service.logs(params).await?;
            to_value(result)
        }
        methods::CANCEL => {
            let params: CancelParams = request.parse_params()?;
            let result: SimpleResult = service.cancel(params).await?;
            to_value(result)
        }
        methods::SHUTDOWN => {
            let result: SimpleResult = service.shutdown().await?;
            to_value(result)
        }
        other => Err(RpcError::method_not_found(other)),
    }
}

fn to_value<T: serde::Serialize>(value: T) -> Result<Value, RpcError> {
    serde_json::to_value(value).map_err(|error| RpcError::internal(error.to_string()))
}

async fn write_response<W: AsyncWrite + Unpin>(
    writer: &mut W,
    response: RpcResponse,
) -> Result<(), ProtocolError> {
    let encoded = serde_json::to_string(&response)?;
    writer.write_all(encoded.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

/// Convenience helper for adapters that need to reject actions they do not
/// implement without taking down the session.
pub fn unsupported_action(action: &Action) -> RpcError {
    RpcError::unsupported(format!(
        "adapter does not support action: {}",
        action.name()
    ))
}
