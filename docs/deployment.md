# Deployment

## Prerequisites

- Rust toolchain (stable)
- eBPF kernel headers
- QUIC TLS certificates (place in `certificates/`)
- Docker (optional, for containerised setup)

## Build

```bash
cargo build --release
```

Build a single workspace member:

```bash
cargo build -p aion-cli
cargo build -p aion-dashboard
```

## Run

```bash
CERT_PATH=./certificates/cert.pem \
KEY_PATH=./certificates/key.pem \
RUST_LOG=info \
cargo run -- csv|metrics|generative
```

For eBPF operations that require elevated privileges:

```bash
CERT_PATH=./certificates/cert.pem \
KEY_PATH=./certificates/key.pem \
RUST_LOG=info \
cargo run -- csv|metrics|generative \
  --config 'target."cfg(all())".runner="sudo -E"'
```

## Docker

Build the image:

```bash
sudo docker build -t aion-agent .
```

Run a container:

```bash
sudo docker run -it --name aaa \
  --cap-add=NET_ADMIN --cap-add=SYS_ADMIN \
  --memory=50M --cpus=1 \
  aion-agent -- busy
```

Monitor container network traffic:

```bash
sudo docker run -it --rm --net=container:aaa \
  nicolaka/netshoot tcpdump -i eth0 -nn -e
```

### macvlan network

Create the network:

```bash
sudo docker network create -d macvlan \
  --subnet=192.168.1.0/24 \
  --gateway=192.168.1.1 \
  -o parent=wlp3s0 mymacvlan
```

Allow host access:

```bash
sudo ip link add link wlp3s0 name host_link type macvlan mode bridge
sudo ip addr add 192.168.1.254/24 dev host_link
sudo ip link set dev host_link up
```

Run with macvlan:

```bash
sudo docker run -it --name lll \
  --cap-add=NET_ADMIN --cap-add=SYS_ADMIN \
  --memory=50M --cpus=1 \
  --network=mymacvlan --ip=192.168.1.70 \
  aion-agent
```

## Tests

```bash
cargo test
```
