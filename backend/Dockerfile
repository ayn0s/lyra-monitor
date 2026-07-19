# syntax=docker/dockerfile:1
FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
COPY . .

RUN cargo build --release -p backend --no-default-features

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --no-create-home --shell /usr/sbin/nologin lyra \
    && mkdir -p /data && chown lyra:lyra /data

COPY --from=builder /workspace/target/release/lyra-backend /usr/local/bin/lyra-backend

USER lyra
WORKDIR /data
ENV RUST_LOG=info
EXPOSE 8080

CMD ["/usr/local/bin/lyra-backend"]
