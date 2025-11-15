FROM rust:1.90-bookworm AS builder   

WORKDIR /app
COPY Cargo.* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY certificates ./certificates
COPY src ./src
COPY src/libebpf.so ./src/libebpf.so
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates python3 iproute2 iputils-ping sudo && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/aion-agent /usr/local/bin/aion-agent
COPY --from=builder /app/certificates /certificates
COPY model-128.json ./
COPY config.toml ./
COPY src/libebpf.so /usr/local/lib/libebpf.so
COPY src/libebpf.so target/bpfel-unknown-none/release/libebpf.so

ENV CERT_PATH="/certificates/cert.pem" KEY_PATH="/certificates/key.pem"
ENV LIBEBPF_PATH=/usr/local/lib/libebpf.so
ENV RUST_LOG=info

EXPOSE 8123

# ENTRYPOINT ["aion-agent"]
ENTRYPOINT ["/tini", "--"]
CMD [ "/bin/sh","-c","aion-agent && python3 m http.server 8123 -bind 0.0.0.0" ]