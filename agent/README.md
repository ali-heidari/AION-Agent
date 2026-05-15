# aixker-Agent

A distributed, ML-driven network orchestration system written in Rust. Part of **Aixker** — a kernel-level RL-driven load routing system for Linux nodes.

## Description

Each agent intercepts incoming requests and uses reinforcement learning (RL) to decide whether to route locally (low load) or redirect to neighbor agents (high load). The system optimizes micro-routing decisions in microseconds, complementing Kubernetes for intra-pod distribution and pre-routing solutions like Nginx/Kong.

## Standards

This project follows the `ai-agent-standards` repository as the base guidance. See `.github/copilot-instructions.md` for project-specific AI agent guidance, and use `ai-agent-standards/README.md` as the source of truth for shared conventions.

## Features

- **Ultra-low latency**: Microsecond-scale kernel-level packet redirection via XDP
- **Lightweight**: ~30MB per agent with minimal resource overhead
- **Decentralized**: Agents autonomously query neighbors for load state
- **Optimized for high-frequency workloads**: Suitable for HFT, robotics, edge AI, real-time simulation

## Prerequisites

- Rust toolchain
- QUIC certificates
- eBPF kernel headers
- Docker (for setup)

## Installation

Clone the repository:

```bash
git clone https://github.com/yourusername/aixker-Agent.git
cd aixker-Agent
```

## Build

Build with eBPF compilation:

```bash
cargo build --release
```

## Usage

Run with specific mode and certificate paths:

```bash
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative
```

For eBPF operations with sudo:

```bash
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative --config 'target."cfg(all())".runner="sudo -E"'
```

### Docker Setup

Build the Docker image:

```bash
sudo docker build -t aixker-agent .
```

Run the container:

```bash
sudo docker run -it --name aaa --cap-add=NET_ADMIN --cap-add=SYS_ADMIN --memory=50M --cpus=1 aixker-agent -- busy
```

Monitor Docker network:

```bash
sudo docker run -it --rm --net=container:aaa nicolaka/netshoot tcpdump -i eth0 -nn -e
```

Add network:

```bash
sudo docker network create -d macvlan --subnet=192.168.1.0/24 --gateway=192.168.1.1 -o parent=wlp3s0 mymacvlan
```

Access from host:

```bash
sudo ip link add link wlp3s0 name host_link type macvlan mode bridge
sudo ip addr add 192.168.1.254/24 dev host_link
sudo ip link set dev host_link up
```

Run container with macvlan network:

```bash
sudo docker run -it --name lll --cap-add=NET_ADMIN --cap-add=SYS_ADMIN --memory=50M --cpus=1 --network=mymacvlan --ip=192.168.1.70 aixker-agent
```

### Docker Compose

`docker-compose.yml` launches 10 agents with static IPs `192.168.1.60–69` on the `mymacvlan` network. Create the macvlan network first (see above) if it doesn't exist.

Build the image:

```bash
docker compose build
```

Start all agents:

```bash
docker compose up -d
```

Start a single agent:

```bash
docker compose up -d agent-60
```

Stop and remove all containers:

```bash
docker compose down
```

View logs:

```bash
docker compose logs -f
docker compose logs -f agent-60
```

## Configuration

Configuration is loaded from `config.toml` with environment variable overrides.

| Key | Purpose | Example |
|-----|---------|---------|
| `mode` | "Infer" or "Train" | `mode="Infer"` |
| `model_name` | Pre-trained model JSON file | `model-128.json` |
| `interval_secs` | eBPF loop polling interval | `interval_secs=10` |
| `batch_size`, `total_batches` | Training parameters | `batch_size=128` |
| `input_number` | Feature vector size | `input_number=6` |
| `output_number` | Action space | `output_number=3` |

## Architecture

- **Distributed agent discovery**: Multicast-based peer discovery
- **ML inference engine**: Uses aixker-rlt library for RL predictions
- **Kernel networking**: eBPF XDP program for packet redirection
- **Async runtime**: Tokio-based task orchestration
- **Decentralized routing**: Gossip protocol for state dissemination

## Testing

Run tests:

```bash
cargo test
```

## Contributing

Contributions are welcome. Please open issues or pull requests.

## Donations

Support this project: [Open Collective](https://opencollective.com/aixkernel)

## License

This project is licensed under the terms in the [LICENSE](LICENSE) file.