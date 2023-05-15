FROM rust:1.69 as builder
ENV CARGO_TERM_COLOR=always \
    CARGO_NET_OFFLINE=false
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN rustup component add clippy
COPY vendor vendor
COPY .cargo .cargo
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN cargo clippy --release --target x86_64-unknown-linux-musl --no-deps -- --deny "warnings"
RUN cargo test --release --target x86_64-unknown-linux-musl

FROM scratch AS runtime
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/jarvis-p1-exporter .
ENTRYPOINT ["./jarvis-p1-exporter"]