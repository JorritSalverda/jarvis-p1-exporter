FROM rust:1.69 as builder
ARG CARGO_BUILD_TARGET=
ENV CARGO_TERM_COLOR=always \
  CARGO_NET_GIT_FETCH_WITH_CLI=true \
  CC_aarch64_unknown_linux_musl=clang \
  AR_aarch64_unknown_linux_musl=llvm-ar \
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-Clink-self-contained=yes -Clinker=rust-lld"
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools clang llvm libudev-dev
RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
ARG CARGO_BUILD_TARGET=
# COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
WORKDIR /app
COPY --from=builder /app/target/${CARGO_BUILD_TARGET}/release/jarvis-p1-exporter .
ENTRYPOINT ["./jarvis-p1-exporter"]