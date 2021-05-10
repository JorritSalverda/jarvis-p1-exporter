# Step 1: Build the application
FROM rust:1.51  as builder
# FROM rust:1.51-alpine  as builder

WORKDIR app

# RUN apk add --update musl-dev
RUN apt-get update && apt-get install -y --no-install-recommends musl-tools
RUN rustup target add x86_64-unknown-linux-musl

COPY . .
RUN cargo test --release --target x86_64-unknown-linux-musl
RUN ls -latr /app/target/x86_64-unknown-linux-musl/release/
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN ls -latr /app/target/x86_64-unknown-linux-musl/release/

# Step 2: Create the runtime container image
FROM scratch AS runtime

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/jarvis-p1-exporter .
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

USER 1000

ENTRYPOINT ["./jarvis-p1-exporter"]