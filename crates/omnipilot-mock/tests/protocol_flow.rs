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

use omnipilot_core::{Action, Selector, Target};
use omnipilot_mock::{mock_handshake, MockAdapter, DEFAULT_TOKEN, DEMO_APP};
use omnipilot_protocol::error::UNAUTHORIZED;
use omnipilot_protocol::{serve_ndjson, ObserveParams, ProtocolClient, ProtocolError};
use tokio::io::duplex;

async fn connect() -> (
    ProtocolClient<
        tokio::io::ReadHalf<tokio::io::DuplexStream>,
        tokio::io::WriteHalf<tokio::io::DuplexStream>,
    >,
    tokio::task::JoinHandle<Result<(), ProtocolError>>,
) {
    let (client_stream, server_stream) = duplex(64 * 1024);
    let (client_read, client_write) = tokio::io::split(client_stream);
    let (server_read, server_write) = tokio::io::split(server_stream);
    let server = tokio::spawn(serve_ndjson(server_read, server_write, MockAdapter::new()));
    (ProtocolClient::new(client_read, client_write), server)
}

#[tokio::test]
async fn handshake_observe_act_and_shutdown() {
    let (mut client, server) = connect().await;
    let (initialize, _, _) = mock_handshake();
    let initialized = client.initialize(initialize).await.expect("initialize");
    assert_eq!(
        initialized.protocol_version,
        omnipilot_core::PROTOCOL_VERSION
    );
    assert_eq!(initialized.device.platform, omnipilot_core::Platform::Mock);

    let observation = client
        .observe(ObserveParams::default())
        .await
        .expect("observe");
    assert!(observation.app.is_none());

    let outcome = client
        .act(Action::LaunchApp {
            app_id: DEMO_APP.to_string(),
        })
        .await
        .expect("launch");
    assert_eq!(outcome.status, omnipilot_core::ActionStatus::Executed);

    let observation = client
        .observe(ObserveParams::default())
        .await
        .expect("observe");
    assert_eq!(
        observation.app.as_ref().map(|app| app.app_id.as_str()),
        Some(DEMO_APP)
    );
    assert!(observation
        .elements
        .iter()
        .any(|element| element.identifier.as_deref() == Some("home.settings")));

    let tap = client
        .act(Action::Tap {
            target: Target::Element {
                selector: Selector::Identifier {
                    value: "home.settings".to_string(),
                },
            },
            times: None,
            duration_ms: None,
        })
        .await
        .expect("tap");
    assert_eq!(tap.status, omnipilot_core::ActionStatus::Executed);

    let logs = client.logs(Some(20)).await.expect("logs");
    assert!(!logs.entries.is_empty());

    client.shutdown().await.expect("shutdown");
    server.await.expect("server join").expect("server result");
}

#[tokio::test]
async fn invalid_token_is_rejected() {
    let (mut client, server) = connect().await;
    let (mut initialize, _, _) = mock_handshake();
    initialize.token = "not-the-token".to_string();

    let error = client
        .initialize(initialize)
        .await
        .expect_err("must reject");
    match error {
        ProtocolError::Rpc(error) => {
            assert_eq!(error.code, UNAUTHORIZED);
            assert!(error.message.contains("token"));
        }
        other => panic!("unexpected error: {other}"),
    }

    drop(client);
    let _ = server.await;
}

#[tokio::test]
async fn methods_before_initialize_are_rejected() {
    let (mut client, server) = connect().await;
    let error = client
        .observe(ObserveParams::default())
        .await
        .expect_err("must reject");
    match error {
        ProtocolError::Rpc(error) => assert_eq!(error.code, UNAUTHORIZED),
        other => panic!("unexpected error: {other}"),
    }

    drop(client);
    let _ = server.await;
}

#[tokio::test]
async fn second_initialize_is_rejected() {
    let (mut client, server) = connect().await;
    let (initialize, _, _) = mock_handshake();
    client.initialize(initialize).await.expect("initialize");
    let (initialize, _, _) = mock_handshake();
    let error = client
        .initialize(initialize)
        .await
        .expect_err("must reject");
    match error {
        ProtocolError::Rpc(error) => assert_eq!(error.code, -32600),
        other => panic!("unexpected error: {other}"),
    }

    drop(client);
    let _ = server.await;
}

#[test]
fn handshake_uses_the_adapter_token() {
    let (initialize, _, token) = mock_handshake();
    assert_eq!(initialize.token, token);
    assert_eq!(token, DEFAULT_TOKEN);
}
