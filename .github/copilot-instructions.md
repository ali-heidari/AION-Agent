# AION-Agent AI Instructions

## Project Overview
AION-Agent is a distributed, ML-driven network orchestration system written in Rust. It's part of **Aixker** — a kernel-level RL-driven load routing system for Linux nodes. Each agent intercepts incoming requests and uses RL to decide: route locally (low load) or redirect to neighbor agents (high load). The system optimizes micro-routing decisions in microseconds, complementing Kubernetes for intra-pod distribution and pre-routing solutions like Nginx/Kong.

**Design Goals:**
- **Ultra-low latency** (microsecond-scale): Kernel-level packet redirection via XDP
- **Lightweight footprint** (~30MB per agent): Minimal resource overhead
- **Decentralized**: Agents autonomously query neighbors for load state
- **High-frequency workloads**: Optimized for HFT, robotics, edge AI, real-time simulation

**Key Architecture:**
- **Distributed agent discovery**: Multicast-based peer discovery ([aion_transporter](src/main.rs#L5))
- **ML inference engine**: Uses aion-rlt (reinforcement learning training) library to predict actions based on system metrics
- **Kernel networking**: eBPF XDP program for packet redirection to load-balanced targets
- **Async runtime**: Tokio-based task orchestration for concurrent agent communication
- **Decentralized routing**: Agents communicate with neighbors to make load-aware redirection decisions

## Critical Build & Run Commands

**Prerequisites**: Requires QUIC certificates and eBPF kernel headers. See [README.md](README.md) for Docker setup.

```bash
# Full build with eBPF compilation
cargo build --release

# Run with specific mode and certificate paths
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative

# With sudo for eBPF operations
CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative --config 'target."cfg(all())".runner="sudo -E"'
```

**Test command**: `cargo test` (see [reward.rs](src/reward.rs#L69) for reward function validation)

## Architecture & Data Flow

**Agent lifecycle** ([main.rs](src/main.rs#L398-L436)):
1. Agent broadcasts itself via multicast with IP, state (0/1/2), and MAC address
2. Agents cache peer metadata in `CACHE` (HashMap)
3. ML model evaluates system metrics → predicts next action (0=high-pressure, 1=normal, 2=low-pressure)
4. Action triggers state change + multicast notification to peers
5. eBPF kernel program redirects packets to agents matching target state

**Key data structures**:
- `Agent` ([main.rs](src/main.rs#L35-L100)): IP identifier, MAC, discrete state, update_time
- `RedirectionData` ([main.rs](src/main.rs#L282-L288)): Kernel eBPF maps local/target IPs and MACs

**Feature extraction** ([mock.rs](src/mock.rs#L100-L150), [reward.rs](src/reward.rs#L1-L23)):
- 6 normalized float features: CPU, memory, swap, disk, throughput, latency
- Range [0.0, 1.0] (clamped); extracted from `SystemMetrics` struct
- Reward function penalizes high load/IO, rewards throughput & low latency
- **Three discrete states** map to actions:
  - `0`: High-pressure (throttle load, redirect requests to neighbors)
  - `1`: Normal operation (local routing)
  - `2`: Low-pressure (accept traffic, scale-up capacity)

## Configuration & Runtime Modes

Config loaded from `config.toml` via [configurations.rs](src/configurations.rs), with environment variable overrides:

| Key | Purpose | Example |
|-----|---------|---------|
| `mode` | "Infer" or "Train" - drives Node.start() behavior | `mode="Infer"` |
| `model_name` | Pre-trained model JSON file for inference | `model-128.json` |
| `interval_secs` | eBPF loop polling interval for agent state changes | `interval_secs=10` |
| `batch_size`, `total_batches` | Training parameters (ignored in Infer mode) | `batch_size=128` |
| `input_number` | Feature vector size (always 6) | `input_number=6` |
| `output_number` | Action space (always 3 states) | `output_number=3` |

**Dataset modes** ([mock.rs](src/mock.rs#L20-L25)):
- `SystemMetrics`: Read live metrics via sysinfo
- `Inputs`: Load from `vmCloud_data.csv` for reproducible testing
- `Generative`: Synthetic state transitions with direction ramping

## Key Patterns & Developer Workflows

**Agent state management**:
- States encoded as `u8`: `0` (high-pressure/throttle), `1` (normal), `2` (low-pressure/scale-up)
- Stored in agent cache with 10-second TTL; stale agents auto-removed during eBPF map updates
- See [main.rs](src/main.rs#L75-L95) for cache eviction logic

**Kernel integration**:
- eBPF program (`libebpf.so`) loaded & attached to XDP hook on default interface
- Three kernel maps: `BLOCKLIST` (IP addr blacklist), `DEVMAP` (interface redirect), `SERVERMAP` (current routing target)
- Query interface name via `ip route show default` command ([main.rs](src/main.rs#L302-L305))

**Error handling pattern**:
- Use `anyhow::Result<T>` for fallible operations
- Most network/system errors logged and continue (graceful degradation)
- Stale agents silently dropped from CACHE on send failure ([main.rs](src/main.rs#L169-L175))

**Testing**:
- Unit tests in `reward.rs` validate reward function edge cases (high/normal/low pressure)
- Mock CSV dataset in `vmCloud_data.csv` for deterministic replay testing
- Run specific test: `cargo test compute_reward --release`

## Cross-Crate Dependencies

- **aion-rlt**: Provides `Node::start()` for inference/training; uses `CONFIG` global
- **aion-transporter**: Multicast listen/send; QUIC client for inter-agent messages
- **aion-math**: Likely used by aion-rlt for neural network ops
- All three are Git dependencies (see [Cargo.toml](Cargo.toml#L6-L8)) — update via Git branch/commit in manifest

**Initialization order** ([main.rs](src/main.rs#L407-L430)):
1. `aion_transporter::quic::init()` — must run before QUIC server/client
2. `send_hello()` + `listen_to_agents()` — multicast bootstrap
3. `aion_rlt::initialize()` inside `start_predicting()` — loads model via CONFIG

## Common Tasks

**Adding a new metric**:
1. Extend `SystemMetrics` struct in [src/metrics/mod.rs](src/metrics/mod.rs)
2. Add extraction to `generate_by_system_metrics()` in [src/mock.rs](src/mock.rs#L100)
3. Update `input_number` in `config.toml` + get_features() return vec in [src/main.rs](src/main.rs#L104)
4. Re-train or adjust reward function in [src/reward.rs](src/reward.rs)

**Debugging agent discovery**:
- Run with `RUST_LOG=debug` to see multicast packets and agent cache updates
- Check `CACHE` contents at runtime via debug print in `represent()` ([main.rs](src/main.rs#L150))
- Verify interface name with: `ip route show default`

**Modifying reward function**:
- Currently returns `(1.0, true)` always ([src/reward.rs](src/reward.rs#L30-L31)) — commented implementation below
- Uncomment lines 32–65 to re-enable adaptive rewards based on system pressure
- Run reward tests: `cargo test test_compute_reward`

## Use Cases & Deployment Contexts

**Target Workloads:**
- **High-frequency trading (HFT)**: Sub-millisecond decision latency for request routing
- **Robotics/autonomous systems**: Real-time sensor processing with adaptive task distribution
- **Edge AI**: Inference workloads distributed across edge nodes based on load
- **Real-time simulation & gaming**: Frame-rate-critical task scheduling across cluster
- **Microservices with high-frequency tasks**: Complements Kubernetes by optimizing inter-pod request flow

**Integration Points:**
- Sits *before* Kubernetes/Nginx: pre-routes requests to reduce bottlenecks
- Works alongside service meshes: decentralized alternative to centralized control planes
- Lightweight enough to co-locate on resource-constrained nodes

## Decentralized Routing Pattern

**Request flow** ([main.rs](src/main.rs#L150-L175)):
1. Agent receives incoming request at kernel (XDP hook)
2. RL model evaluates local `SystemMetrics` → predicts action (0/1/2)
3. If action=0 (high-pressure): Query cached neighbor agent loads
4. Redirect packet to neighbor IP via eBPF `SERVERMAP` kernel map
5. Response returns to originating agent → client

**Neighbor communication**:
- Agents broadcast state (0/1/2) via multicast on 10-second interval
- Cache (`CACHE` HashMap) stores peer metadata: IP, MAC, state, last-update timestamp
- Stale agents (>10s old) auto-evicted; ping verification removes unreachable nodes
- No central coordinator needed — pure gossip protocol for state dissemination
