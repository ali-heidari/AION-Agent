# AION-Agent

A distributed, ML-driven network orchestration system written in Rust. Part of **Aixker** — a kernel-level RL-driven load routing system for Linux nodes.

Each agent intercepts incoming requests and uses reinforcement learning to decide whether to route locally or redirect to a neighbor, optimizing micro-routing decisions at microsecond scale via eBPF/XDP.

## Features

- **Ultra-low latency** — microsecond-scale kernel-level packet redirection via XDP
- **Lightweight** — ~30MB per agent with minimal resource overhead
- **Decentralized** — agents autonomously query neighbors for load state
- **Observability** — Prometheus metrics on `:9090`, dashboard on `:8080`

## Quick start

```bash
git clone https://github.com/yourusername/AION-Agent.git
cd AION-Agent
cargo build --release
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- metrics
```

### Test - benchmark - monitoring

For running 10 containers and monitoring, you can use this command from the root:

```bash
sudo docker compose -f  dashboard/docker-compose.yml -f agent/docker-compose.yml up -d
```

## Documentation

See [`docs/index.md`](docs/index.md) for the full documentation index.

| Document | Description |
| -------- | ----------- |
| [Architecture](docs/architecture.md) | System overview, components, and subsystems |
| [Configuration](docs/configuration.md) | `config.toml` reference and environment variables |
| [Deployment](docs/deployment.md) | Build, run, Docker, and test instructions |

## Standards

This project follows [`ai-agent-standards`](https://github.com/ali-heidari/ai-agent-standards). See [`.agent/agent-instructions.md`](.agent/agent-instructions.md) for project-specific AI agent guidance.

## Contributing

Contributions are welcome. Please open issues or pull requests.

## Donations

Support this project: [Open Collective](https://opencollective.com/aixkernel)

## License

This project is licensed under the terms in the [LICENSE](LICENSE) file.
