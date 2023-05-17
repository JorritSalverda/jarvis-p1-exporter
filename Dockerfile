FROM rust:1.69 as builder
ARG CARGO_BUILD_TARGET=
ENV CARGO_TERM_COLOR=always \
  CARGO_NET_GIT_FETCH_WITH_CLI=true
WORKDIR /app
RUN apt-get update && apt-get install -y libudev-dev
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
RUN rustup component add clippy
COPY . .
RUN cargo build --release
RUN cargo clippy --release --no-deps -- --deny "warnings"
RUN cargo test --release

FROM debian:bullseye-slim AS runtime
ARG CARGO_BUILD_TARGET=
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
WORKDIR /app
COPY --from=builder /app/target/${CARGO_BUILD_TARGET}/release/jarvis-p1-exporter .
ENTRYPOINT ["./jarvis-p1-exporter"]