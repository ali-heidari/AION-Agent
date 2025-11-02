FROM rust:1.90-bookworm AS builder   

WORKDIR /app
COPY Cargo.* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aion-agent /usr/local/bin/aion-agent


ENTRYPOINT ["aion-agent"]