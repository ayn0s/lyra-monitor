use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use shared::pb::terminal_input::Payload;
use shared::pb::{AuthCredentials, PtyResize, TerminalInput};
use shared::AgentServiceClient;
use std::time::Duration;
use tonic::transport::Channel;
use tracing::{debug, warn};

async fn send_error(ws_tx: &mut SplitSink<WebSocket, Message>, message: &str) {
    let _ = futures::SinkExt::send(ws_tx, Message::Text(format!("[lyra] {message}"))).await;
    // Gives a WebSocket proxy in front of the backend (e.g. nginx) time to
    // relay the frame to the client before the underlying connection drops.
    tokio::time::sleep(Duration::from_millis(500)).await;
}

pub async fn bridge(socket: WebSocket, mut client: AgentServiceClient<Channel>) {
    let (mut ws_tx, mut ws_rx) = futures::StreamExt::split(socket);

    let auth = match futures::StreamExt::next(&mut ws_rx).await {
        Some(Ok(Message::Text(text))) => parse_auth(&text),
        _ => None,
    };
    let Some(auth) = auth else {
        send_error(&mut ws_tx, "expected credentials as first message").await;
        return;
    };

    let (grpc_in_tx, grpc_in_rx) = tokio::sync::mpsc::channel::<TerminalInput>(64);
    let outbound = tokio_stream::wrappers::ReceiverStream::new(grpc_in_rx);

    if grpc_in_tx.send(auth).await.is_err() {
        return;
    }

    let response = match client.stream_terminal(outbound).await {
        Ok(resp) => resp,
        Err(err) => {
            warn!(error = %err, "failed to open gRPC stream to agent");
            send_error(&mut ws_tx, err.message()).await;
            return;
        }
    };
    let mut inbound = response.into_inner();

    let ws_to_grpc = tokio::spawn(async move {
        while let Some(Ok(msg)) = futures::StreamExt::next(&mut ws_rx).await {
            let input = match msg {
                Message::Binary(bytes) => Some(TerminalInput {
                    payload: Some(Payload::Stdin(bytes)),
                }),
                Message::Text(text) => parse_resize(&text),
                Message::Close(_) => break,
                _ => None,
            };

            if let Some(input) = input {
                if grpc_in_tx.send(input).await.is_err() {
                    break;
                }
            }
        }
        debug!("browser -> agent stream ended");
    });

    let grpc_to_ws = tokio::spawn(async move {
        while let Some(result) = futures::StreamExt::next(&mut inbound).await {
            let chunk = match result {
                Ok(chunk) => chunk,
                Err(err) => {
                    send_error(&mut ws_tx, err.message()).await;
                    break;
                }
            };

            if futures::SinkExt::send(&mut ws_tx, Message::Binary(chunk.stdout))
                .await
                .is_err()
            {
                break;
            }
        }
        debug!("agent -> browser stream ended");
    });

    let _ = tokio::join!(ws_to_grpc, grpc_to_ws);
}

fn parse_auth(text: &str) -> Option<TerminalInput> {
    #[derive(serde::Deserialize)]
    struct AuthMsg {
        #[serde(rename = "type")]
        kind: String,
        username: String,
        password: String,
    }

    let msg: AuthMsg = serde_json::from_str(text).ok()?;
    if msg.kind != "auth" {
        return None;
    }

    Some(TerminalInput {
        payload: Some(Payload::Auth(AuthCredentials {
            username: msg.username,
            password: msg.password,
        })),
    })
}

fn parse_resize(text: &str) -> Option<TerminalInput> {
    #[derive(serde::Deserialize)]
    struct ResizeMsg {
        #[serde(rename = "type")]
        kind: String,
        cols: u32,
        rows: u32,
    }

    let msg: ResizeMsg = serde_json::from_str(text).ok()?;
    if msg.kind != "resize" {
        return None;
    }

    Some(TerminalInput {
        payload: Some(Payload::Resize(PtyResize {
            cols: msg.cols,
            rows: msg.rows,
        })),
    })
}
