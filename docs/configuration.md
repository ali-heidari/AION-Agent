# Configuration

Runtime configuration is loaded from `config.toml` at startup. Environment variables `CERT_PATH` and `KEY_PATH` are required.

## config.toml reference

| Key | Type | Purpose | Example |
|-----|------|---------|---------|
| `mode` | string | `"Infer"` or `"Train"` | `mode = "Infer"` |
| `model_name` | string | Path to pre-trained model JSON | `model_name = "model-128.json"` |
| `interval_secs` | integer | eBPF loop polling interval in seconds | `interval_secs = 10` |
| `batch_size` | integer | Training batch size | `batch_size = 128` |
| `total_batches` | integer | Total training batches | `total_batches = 1000` |
| `input_number` | integer | Feature vector size | `input_number = 6` |
| `output_number` | integer | Action space size | `output_number = 3` |

## Environment variables

| Variable | Purpose |
|----------|---------|
| `CERT_PATH` | Path to QUIC TLS certificate (`cert.pem`) |
| `KEY_PATH` | Path to QUIC TLS private key (`key.pem`) |
| `RUST_LOG` | Log level (`error`, `warn`, `info`, `debug`, `trace`) |

## Run modes

Pass the mode as a positional argument when running the agent:

| Argument | Description |
|----------|-------------|
| `csv` | Load feature data from a CSV file |
| `metrics` | Use live Prometheus metrics as input |
| `generative` | Generate synthetic input data |
