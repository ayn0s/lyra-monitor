use axum::extract::ws::{Message, WebSocket};
use shared::pb::terminal_input::Payload;
use shared::pb::{PtyResize, TerminalInput};
use shared::AgentServiceClient;
use tonic::transport::Channel;
use tracing::{debug, warn};

pub async fn bridge(socket: WebSocket, mut client: AgentServiceClient<Channel>) {
    let (mut ws_tx, mut ws_rx) = futures::StreamExt::split(socket);

    let (grpc_in_tx, grpc_in_rx) = tokio::sync::mpsc::channel::<TerminalInput>(64);
    let outbound = tokio_stream::wrappers::ReceiverStream::new(grpc_in_rx);

    let response = match client.stream_terminal(outbound).await {
        Ok(resp) => resp,
        Err(err) => {
            warn!(error = %err, "failed to open gRPC stream to agent");
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
        while let Some(Ok(chunk)) = futures::StreamExt::next(&mut inbound).await {
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
