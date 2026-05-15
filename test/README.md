# Tests

## Getting started

### Containers with AIxKer agents

```bash
sudo docker compose -f  dashboard/docker-compose.yml -f agent/docker-compose.yml -d
```

### Containers with Nginx/HAproxy

```bash
sudo docker compose -f  dashboard/docker-compose.yml -f test/docker-compose.yml up -d
```

## Benchmark

```bash
k6 run k6-v2.js                        # all 10 containers (default)
k6 run --env TARGET=nginx   k6-v2.js   # through nginx (192.168.1.81:80)
k6 run --env TARGET=haproxy k6-v2.js   # through haproxy (192.168.1.80:80)
k6 run --env TARGET=all     k6-v2.js   # direct to each container :8123
```
