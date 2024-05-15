FROM rust:1.78.0 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:12.5-slim
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/main /usr/local/bin/telemetry-service
ENTRYPOINT ["telemetry-service"]
