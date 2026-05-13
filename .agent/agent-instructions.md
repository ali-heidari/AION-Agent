# AION-Agent — AI Agent Instructions

Base standard: [`instructions.md`](https://github.com/ali-heidari/ai-agent-standards/blob/main/instructions.md) from [ai-agent-standards](https://github.com/ali-heidari/ai-agent-standards).

## Project purpose

AION-Agent is a distributed, ML-driven network orchestration system written in Rust. Each agent intercepts incoming requests and uses reinforcement learning (RL) to decide whether to route locally or redirect to a neighbor. It targets microsecond-scale routing for high-frequency workloads via kernel-level eBPF/XDP.

## Key files

| Path | Purpose |
| ---- | ------- |
| `agent/` (`aion-agent`) | Core RL agent — eBPF/XDP, Prometheus metrics on `:9090` |
| `cli/` (`aion-cli`) | CLI to query agent metrics |
| `dashboard/` (`aion-dashboard`) | Axum HTTP dashboard on `:8080`, proxies agent metrics |
| `Cargo.toml` | Workspace root — members: `agent`, `cli`, `dashboard` |
| `config.toml` | Runtime config (mode, model, intervals, feature sizes) |
| `certificates/` | QUIC TLS certs required at runtime |

## Build commands

Only use build steps that exist in this repo. The canonical build tool is `cargo`.

```bash
cargo build                          # all workspace members
cargo build -p aion-cli              # CLI only
cargo build -p aion-dashboard        # dashboard only
cargo run -- csv|metrics|generative  # run agent (set CERT_PATH and KEY_PATH)
```

Do not invent build or CI steps unless an explicit manifest or build file (e.g. `Makefile`, `.github/workflows/`) exists in this repo.

## Commit conventions

Format: `type(scope): short summary`

Types: `feat`, `fix`, `chore`, `docs`, `ci`, `style`, `refactor`, `test`

## Assistant interaction protocol

1. State the overall task before starting work.
2. Explain each step clearly and concisely.
3. Before writing code, describe the exact code change planned.
4. After describing a change, ask whether the developer understands or agrees before applying it.
5. Once confirmed, apply the code and proceed to the next step.
6. If the user says "sync instructions", reapply `instructions.md` from [ai-agent-standards](https://github.com/ali-heidari/ai-agent-standards) to this file.
7. This file (`.agent/agent-instructions.md`) is the master instruction file. All other agent instruction files must be placed inside `.agent/`.

## How to use these standards

**Option A — Copy files:** Copy `instructions.md` and the convention markdown files (`code-conventions.md`, `commit-conventions.md`, `cicd-conventions.md`, `docs-conventions.md`) from [ai-agent-standards](https://github.com/ali-heidari/ai-agent-standards) into the project root. Update this file to reflect any local overrides.

**Option B — Submodule:** Add ai-agent-standards as a git submodule:

```bash
git submodule add https://github.com/ali-heidari/ai-agent-standards ai-agent-standards
```

Reference the canonical files from there and keep `.agent/agent-instructions.md` for local overrides.
