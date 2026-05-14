# Aixker Dashboard

Observability stack for the Aixker cluster. Runs as a single instance and provides a unified view of all containers in the environment.

## Architecture

```text
┌──────────────────────────────────────────────────────────────┐
│         Control plane  (one instance, anywhere in network)    │
│                                                              │
│   cAdvisor   (:8080)  ←  reads Docker cgroups directly      │
│   Prometheus (:9091)  ←  scrapes cAdvisor                   │
│   Grafana    (:3000)  ←  queries Prometheus, renders panels  │
└──────────────────────────────────────────────────────────────┘
```

The Aixker agent runs inside the Linux kernel via eBPF/XDP on each machine. It makes microsecond-scale routing decisions before the OS network stack processes any packet. It exposes no web server and listens on no port — it is invisible to user space.

## Node roles

### cAdvisor

Reads container resource usage directly from Docker's cgroups on the host — no instrumentation inside containers needed. Exposes per-container CPU, memory, network, and filesystem metrics in Prometheus format on `:8080`.

### Prometheus

Scrapes cAdvisor on a 5-second interval and stores the time-series data. Answers PromQL queries from Grafana.

Port: `9091` (host) → `9090` (container)

### Grafana

Queries Prometheus and renders the container dashboard. Pre-provisioned with the **Aixker Cluster** dashboard — no manual configuration needed after startup.

Port: `3000`

---

## Setup

### 1. Prerequisites

- Docker Engine 25.x or later
- Docker Compose v2

### 2. Start the stack

From the `dashboard/` directory:

```bash
docker compose up -d
```

This starts Prometheus, Grafana, and cAdvisor together.

### 3. Open Grafana

Navigate to `http://localhost:3000`.

Default credentials: `admin` / `admin`. Change the password on first login.

The **Aixker Cluster** dashboard loads automatically.

---

## Stopping the stack

```bash
docker compose down
```

To also remove stored Grafana data:

```bash
docker compose down -v
```

---

## Dashboard panels

| Panel | Description |
| ----- | ----------- |
| Running Containers | Count of active containers |
| Total CPU Usage | Aggregate CPU % across all containers |
| Total Memory Usage | Aggregate working set memory |
| Total Network RX / TX | Aggregate throughput |
| CPU Usage per Container | Time series per container |
| Memory Usage per Container | Working set per container |
| Memory Limit per Container | Configured limit per container |
| Network RX / TX per Container | Per-container throughput |
| Filesystem Usage per Container | Disk used per container |
| Container Overview | Sortable table — CPU% and memory snapshot |

## Metric reference

Metrics are emitted by cAdvisor using the `container_` prefix. System cgroups are filtered out — only containers with a Docker image appear in the dashboard.

| Metric | Type | Description |
| ------ | ---- | ----------- |
| `container_cpu_usage_seconds_total` | counter | Cumulative CPU time consumed |
| `container_memory_working_set_bytes` | gauge | Current working set memory |
| `container_spec_memory_limit_bytes` | gauge | Configured memory limit |
| `container_network_receive_bytes_total` | counter | Cumulative network bytes received |
| `container_network_transmit_bytes_total` | counter | Cumulative network bytes transmitted |
| `container_fs_usage_bytes` | gauge | Filesystem bytes consumed |
