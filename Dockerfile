# syntax=docker/dockerfile:1.3
FROM harbor.crungo.net/docker-proxy/library/rust:1.88-alpine as chef
ARG RUST_TARGET=x86_64-unknown-linux-musl
RUN apk add --no-cache musl-dev build-base openssl-dev && rustup target add $RUST_TARGET
WORKDIR /app
RUN cargo install cargo-chef

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder
ARG RUST_TARGET=x86_64-unknown-linux-musl
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target $RUST_TARGET --recipe-path recipe.json
COPY . .
RUN cargo build --release --target $RUST_TARGET

FROM harbor.crungo.net/docker-proxy/library/alpine:latest AS runtime
ARG RUST_TARGET=x86_64-unknown-linux-musl
COPY --from=builder /app/target/$RUST_TARGET/release/treat-dispenser-api /usr/local/bin/treat-dispenser-api
ENTRYPOINT ["/usr/local/bin/treat-dispenser-api"]

FROM scratch as binary-export
ARG RUST_TARGET=x86_64-unknown-linux-musl
COPY --from=runtime /usr/local/bin/treat-dispenser-api /treat-dispenser-api
ENTRYPOINT ["/treat-dispenser-api"]