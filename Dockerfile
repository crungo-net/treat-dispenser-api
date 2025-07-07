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

# Add a build arg that changes on every build to bust the cache
ARG CACHE_BUST=unknown
# Force cache invalidation with a dummy command that changes on every build
RUN echo "Cache bust: ${CACHE_BUST}" > /tmp/cache_bust


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
