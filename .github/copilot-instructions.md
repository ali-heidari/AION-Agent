# AION-Agent — Copilot instructions

Purpose: Give AI coding agents the essential, actionable knowledge to be productive in this repo.

Big picture
- Distributed, RL-driven network orchestration system written in Rust.
- Uses reinforcement learning and eBPF/XDP packet handling to make low-latency routing decisions across nodes.

Key files & where to look
- `src/main.rs`: Application entry point and runtime orchestration
- `src/configurations.rs`: Configuration loading and runtime parameters
- `src/mock.rs`: Synthetic and system-metric dataset support
- `src/reward.rs`: RL reward logic
- `Cargo.toml`: Rust dependencies and build configuration
- `README.md`: Repo overview and usage instructions

Base standards
- This project adopts the `ai-agent-standards` repository as the base standard.
- Use `ai-agent-standards/instructions.md` as the canonical base guidance.
- The base `instructions.md` links to the shared conventions in `code-conventions.md`, `cicd-conventions.md`, `commit-conventions.md`, `docs-conventions.md`, and `repository.md`.

Core rule
- Do not invent build or CI steps unless explicit manifest or build files exist in this repo (for example `Cargo.toml`, `Makefile`, or `pyproject.toml`).

How to use these standards
- Option 1: copy the relevant `ai-agent-standards` files into this repo root.
- Option 2: reference `ai-agent-standards` as a git submodule or shared template, and keep a small local README note describing the source of truth.

Build & run (reproducible commands)
- Build: `cargo build --release`
- Run: `CERT_PATH=./certificates/cert.pem KEY_PATH=./certificates/key.pem RUST_LOG=info cargo run -- csv|metrics|generative`

Project-specific conventions (do not change silently)
- Action space: 0 (high-pressure/throttle), 1 (normal), 2 (low-pressure/scale-up)
- Feature dimensions: 6 normalized metrics (CPU, memory, swap, disk, throughput, latency)
- Reward range: [-1.0, 1.0] clamped
- Model persistence: JSON serialization with NaN sanitization

Integration points & expectations
- Uses aion-rlt for RL components
- Requires QUIC certificates and eBPF kernel headers
- Async Tokio runtime with background training worker

Safe edits checklist for feature changes
- When modifying action space: Update OUTPUT_DIM and reward mapping
- When adding features: Update INPUT_DIM and feature extraction
- When changing reward function: Ensure [-1.0, 1.0] clamping

Quick debugging tips
- Use RUST_LOG=debug for detailed training logs
- Check model.json for NaN values after training
- Test reward function with cargo test -- --nocapture
