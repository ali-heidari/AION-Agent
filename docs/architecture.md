# Architecture

## Overview

AION-Agent is a distributed, ML-driven network orchestration system. Each node runs an independent agent that intercepts incoming requests and uses reinforcement learning to make microsecond-scale routing decisions.

## Components

| Component | Crate | Description |
|-----------|-------|-------------|
| Core agent | `agent/` (`aion-agent`) | eBPF/XDP packet interception, RL inference, Prometheus metrics on `:9090` |
| CLI | `cli/` (`aion-cli`) | Query agent metrics from the command line |
| Dashboard | `dashboard/` (`aion-dashboard`) | Axum HTTP dashboard on `:8080`, proxies agent metrics |

## Routing logic

The agent observes local CPU/network load as a feature vector and feeds it into the RL model (`aixker_rlt`). The model outputs one of three actions:

- **Route locally** — serve the request on this node
- **Redirect to neighbor** — forward to a peer with lower load
- **Drop / queue** — defer the request

## Subsystems

- **Peer discovery** — multicast-based; agents announce themselves and track neighbor load state via gossip protocol
- **ML inference engine** — uses the `aixker_rlt` library for RL predictions; model weights loaded from a JSON file (`model-128.json` by default)
- **Kernel networking** — eBPF XDP program compiled and attached at startup for microsecond-level packet redirection
- **Async runtime** — Tokio task orchestration for concurrent I/O, metrics, and inference loops
- **Observability** — Prometheus metrics on `:9090/metrics`; human-readable dashboard on `:8080`

## Workspace layout

```
AION-Agent/
├── agent/          # aion-agent crate
├── cli/            # aion-cli crate
├── dashboard/      # aion-dashboard crate
├── certificates/   # QUIC TLS certs (required at runtime)
├── config.toml     # runtime configuration
└── Cargo.toml      # workspace root
```
