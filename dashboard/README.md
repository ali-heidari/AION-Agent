# Aixker Dashboard

Observability stack for the Aixker cluster. Runs as a single master instance and provides a unified view across all N agents in the environment.

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│           Control plane  (one instance, anywhere in network)     │
│                                                                 │
│   Prometheus (:9091)  ←  scrapes all agents + receives OTel    │
│   Grafana    (:3000)  ←  queries Prometheus, renders panels     │
└────────────────────────────┬────────────────────────────────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
          ▼                  ▼                  ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   Machine 1     │ │   Machine 2     │ │   Machine N     │
│                 │ │                 │ │                 │
│  Aixker agent   │ │  Aixker agent   │ │  Aixker agent   │
│  (kernel/XDP)   │ │  (kernel/XDP)   │ │  (kernel/XDP)   │
│  :9090/metrics  │ │  :9090/metrics  │ │  :9090/metrics  │
│                 │ │                 │ │                 │
│  Docker daemon  │ │  Docker daemon  │ │  Docker daemon  │
│  (built-in OTel)│ │  (built-in OTel)│ │  (built-in OTel)│
│  OTLP push ─────┼─┼─────────────────┼─┼──▶ Prometheus   │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## Node roles

### Aixker agent (one per machine)

Runs inside the Linux kernel via eBPF/XDP. It intercepts raw packets before the OS network stack processes them. It reads machine metrics (CPU, memory, network load), feeds them into the embedded RL model (<5 KB), and in microseconds decides whether this node handles the request or redirects it to a neighbor. Agents communicate with each other via multicast gossip to share load state. The application in user space is never aware.

Exposes Prometheus metrics at `http://<node-ip>:9090/metrics`. Prometheus scrapes this endpoint every 5 seconds.

### Docker daemon (built-in OTel, one per machine)

Since Docker Engine 25.x, the Docker daemon has a built-in OpenTelemetry exporter. It pushes container-level metrics (CPU usage, memory, network I/O per container) via OTLP directly to Prometheus, which accepts OTLP natively since version 2.47. No sidecar or separate collector process is needed.

Pushes to `http://<control-plane-ip>:9091/api/v1/otlp/v1/metrics`.

### Prometheus (one instance)

The central time-series database. It has two ingest paths:

- **Pull** — scrapes each Aixker agent's `:9090/metrics` endpoint on a 5-second interval
- **Push** — receives OTLP from each Docker daemon via `--web.enable-otlp-receiver`

Both data sources are stored in the same database and queryable together in Grafana.

### Grafana (one instance)

Queries Prometheus and renders the cluster dashboard. Pre-provisioned with the Aixker Cluster dashboard showing active agent count, routing decisions per node, CPU load gauges, RL inference latency, and Docker container metrics. No manual configuration needed after startup.

---

## Setup

### 1. Prerequisites

- Docker Engine 25.x or later with Docker Compose v2
- Machines running the Aixker agent must be reachable on port `9090` from the control plane
- Machines running Docker must be able to reach the control plane on port `9091`

### 2. Configure agent targets

Edit [`prometheus/prometheus.yml`](prometheus/prometheus.yml) and replace the placeholder values with your actual agent IPs:

```yaml
static_configs:
  - targets:
      - "192.168.1.10:9090"
      - "192.168.1.11:9090"
      - "192.168.1.12:9090"
```

Add one entry per machine running an Aixker agent.

### 3. Start the stack

From the `dashboard/` directory:

```bash
docker compose up -d
```

### 4. Open Grafana

Navigate to `http://localhost:3000`.

Default credentials: `admin` / `admin`. Change the password on first login.

The **Aixker Cluster** dashboard loads automatically.

---

## Configuring Docker's built-in OpenTelemetry

Docker 25.x ships with a built-in OTel exporter in the daemon. No extra process is needed — just configure `/etc/docker/daemon.json` on each machine and restart Docker.

### Step 1 — Edit daemon config

On each machine that runs Docker containers, open `/etc/docker/daemon.json` (create it if it does not exist):

```json
{
  "metrics-addr": "127.0.0.1:9323"
}
```

Replace `<CONTROL_PLANE_IP>` with the IP or hostname of the machine running this dashboard stack.

### Step 2 — Restart Docker

```bash
sudo systemctl restart docker
```

### Step 3 — Verify

Docker container metrics will appear in Grafana's **Docker Container Metrics** panel within one scrape cycle. You can also verify by querying Prometheus directly:

```text
http://localhost:9091/api/v1/query?query=docker_container_cpu_usage_seconds_total
```

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

## Metric reference

Metrics emitted by each Aixker agent:

| Metric | Type | Description |
| ------ | ---- | ----------- |
| `aixker_redirected_requests_total` | counter | Requests redirected to a neighbor node |
| `aixker_local_requests_total` | counter | Requests handled locally on this node |
| `aixker_inference_latency_microseconds` | gauge | RL model inference time in microseconds |
| `aixker_cpu_load_percent` | gauge | Current CPU load on this node |

If metric names differ in your build, update the PromQL expressions in [`grafana/dashboards/aixker-cluster.json`](grafana/dashboards/aixker-cluster.json).
