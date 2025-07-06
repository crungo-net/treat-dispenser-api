# syntax=docker/dockerfile:1.3

FROM rust:1.88-alpine as builder

RUN apk add --no-cache musl-dev build-base openssl-dev

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {println!("DUMMY");}' > src/main.rs 
RUN cargo build --release --target x86_64-unknown-linux-musl 

# this will prevent the dummy binary from being used in the final image
RUN rm -rf target/x86_64-unknown-linux-musl/release/treat-dispenser-api && rm -rf src

# Build application runtime image
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:latest as runtime
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/treat-dispenser-api /usr/local/bin/treat-dispenser-api

ENTRYPOINT ["/usr/local/bin/treat-dispenser-api"]

# Export binary output
FROM scratch as binary-export
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/treat-dispenser-api /
ENTRYPOINT ["/treat-dispenser-api"]
