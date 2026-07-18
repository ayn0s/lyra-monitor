use crate::{metrics, metrics_history::MetricsHistory, pam_auth, pty::PtySession, systemd};
use futures::Stream;
use shared::pb::agent_service_server::AgentService;
use shared::pb::terminal_input::Payload;
use shared::pb::{
    GetMetricsHistoryRequest, GetMetricsHistoryResponse, ListServicesRequest, ListServicesResponse,
    MetricsRequest, MetricsResponse, MetricsSample, PingRequest, PingResponse, ServiceUnit,
    TerminalInput, TerminalOutput,
};
use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};
use tonic::{Request, Response, Status, Streaming};

pub struct AgentServiceImpl {
    pub history: MetricsHistory,
}

type TerminalStream = Pin<Box<dyn Stream<Item = Result<TerminalOutput, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl AgentService for AgentServiceImpl {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let nonce = request.into_inner().nonce;
        let server_time_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Ok(Response::new(PingResponse {
            nonce,
            server_time_unix_ms,
        }))
    }

    async fn get_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        let raw = tokio::task::spawn_blocking(metrics::snapshot)
            .await
            .map_err(|e| Status::internal(format!("metrics task cancelled: {e}")))?
            .map_err(|e| Status::internal(format!("failed to read metrics: {e}")))?;

        Ok(Response::new(MetricsResponse {
            cpu_usage_percent: raw.cpu_usage_percent,
            mem_used_bytes: raw.mem_used_bytes,
            mem_total_bytes: raw.mem_total_bytes,
            load_average_1m: raw.load_average_1m,
            uptime_seconds: raw.uptime_seconds,
        }))
    }

    async fn get_metrics_history(
        &self,
        _request: Request<GetMetricsHistoryRequest>,
    ) -> Result<Response<GetMetricsHistoryResponse>, Status> {
        let history = self.history.read().await;
        let samples = history
            .iter()
            .map(|entry| MetricsSample {
                timestamp_unix_ms: entry.timestamp_unix_ms,
                cpu_usage_percent: entry.raw.cpu_usage_percent,
                mem_used_bytes: entry.raw.mem_used_bytes,
                mem_total_bytes: entry.raw.mem_total_bytes,
                load_average_1m: entry.raw.load_average_1m,
            })
            .collect();

        Ok(Response::new(GetMetricsHistoryResponse { samples }))
    }

    async fn list_services(
        &self,
        _request: Request<ListServicesRequest>,
    ) -> Result<Response<ListServicesResponse>, Status> {
        let services = systemd::list_services()
            .await
            .map_err(|e| Status::internal(format!("failed to list services: {e}")))?
            .into_iter()
            .map(|unit| ServiceUnit {
                name: unit.name,
                description: unit.description,
                load_state: unit.load_state,
                active_state: unit.active_state,
                sub_state: unit.sub_state,
            })
            .collect();

        Ok(Response::new(ListServicesResponse { services }))
    }

    type StreamTerminalStream = TerminalStream;

    async fn stream_terminal(
        &self,
        request: Request<Streaming<TerminalInput>>,
    ) -> Result<Response<Self::StreamTerminalStream>, Status> {
        let mut inbound = request.into_inner();

        let first = inbound
            .message()
            .await
            .map_err(|e| Status::internal(format!("failed to read terminal input: {e}")))?
            .ok_or_else(|| Status::invalid_argument("expected credentials as first message"))?;

        let Some(Payload::Auth(creds)) = first.payload else {
            return Err(Status::invalid_argument(
                "first message must be authentication credentials",
            ));
        };

        let username = creds.username;
        let password = creds.password;
        tokio::task::spawn_blocking({
            let username = username.clone();
            move || pam_auth::authenticate(&username, &password)
        })
        .await
        .map_err(|e| Status::internal(format!("authentication task cancelled: {e}")))?
        .map_err(|e| Status::unauthenticated(format!("{e}")))?;

        let session = PtySession::spawn(&username)
            .map_err(|e| Status::internal(format!("failed to create PTY: {e}")))?;
        let PtySession {
            stdin_tx,
            resize_tx,
            mut stdout_rx,
        } = session;

        tokio::spawn(async move {
            while let Ok(Some(input)) = inbound.message().await {
                match input.payload {
                    Some(Payload::Stdin(bytes)) => {
                        let _ = stdin_tx.send(bytes);
                    }
                    Some(Payload::Resize(resize)) => {
                        let _ = resize_tx.send((resize.cols as u16, resize.rows as u16));
                    }
                    _ => {}
                }
            }
        });

        let (out_tx, out_rx) = tokio::sync::mpsc::channel::<Result<TerminalOutput, Status>>(64);
        tokio::spawn(async move {
            while let Some(chunk) = stdout_rx.recv().await {
                if out_tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
            }
        });

        let stream = tokio_stream::wrappers::ReceiverStream::new(out_rx);
        Ok(Response::new(Box::pin(stream) as Self::StreamTerminalStream))
    }
}
