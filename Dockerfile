# Target platform triple. Leave unset to autodetect.
# ARG CARGO_BUILD_TARGET=

# Set to true if using vendored sources
ARG CARGO_NET_OFFLINE=false

FROM rust:1.69 as builder

ENV CARGO_TERM_COLOR=always
# CARGO_NET_OFFLINE=false

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends musl-tools
# RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
RUN rustup component add clippy

# COPY vendor vendor
# COPY .cargo .cargo
COPY . .

# RUN cat .cargo/config.toml
# RUN cat Cargo.toml
# RUN cat Cargo.lock
# RUN cargo tree

RUN cargo build --release
RUN cargo clippy --release --no-deps -- --deny "warnings"
RUN cargo test --release

FROM debian:bullseye-slim AS runtime
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/jarvis-p1-exporter .
ENTRYPOINT ["./jarvis-p1-exporter"]