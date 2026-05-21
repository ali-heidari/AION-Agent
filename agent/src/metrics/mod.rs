use anyhow::Result;
use core::f32;
use std::ops::Div;
use std::path::Path;
use std::time::Instant;

const MEMORY_THRESHOLD: f32 = 0.8;
// 2 MB/s rx rate maps to throughput=0.0 (fully saturated). Tune if needed.
const MAX_RX_BPS: f32 = 2_000_000.0;
// 100 concurrent TCP connections to port 8123 maps to latency=1.0.
const MAX_CONNECTIONS: f32 = 100.0;

#[derive(Clone, Debug, PartialEq)]
pub enum DatasetMode {
    SystemMetrics,
    Inputs,
    Generative,
}

#[derive(Debug, Clone)]
pub struct SyntheticState {
    pub cpu: f32,
    pub mem: f32,
    pub swap: f32,
    pub disk: f32,
    pub throughput: f32,
    pub latency: f32,
    pub direction: f32,
    pub count: u32,
    records: Vec<Vec<f32>>,
    mode: DatasetMode,
    last_cpu_usec: u64,
    last_rx_bytes: u64,
    last_update: Instant,
}

// cgroup v2: /sys/fs/cgroup/cpu.stat usage_usec
// cgroup v1 fallback: /sys/fs/cgroup/cpu/cpuacct.usage (nanoseconds → µs)
fn read_cgroup_cpu_usec() -> u64 {
    if let Ok(content) = std::fs::read_to_string("/sys/fs/cgroup/cpu.stat") {
        for line in content.lines() {
            if line.starts_with("usage_usec ") {
                return line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }
    }
    std::fs::read_to_string("/sys/fs/cgroup/cpu/cpuacct.usage")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0)
        / 1000
}

// Returns (used_bytes, limit_bytes). Returns (0, 0) if unreadable.
fn read_cgroup_memory() -> (u64, u64) {
    // cgroup v2
    let current = std::fs::read_to_string("/sys/fs/cgroup/memory.current")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    let max = std::fs::read_to_string("/sys/fs/cgroup/memory.max")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    if max > 0 && max < u64::MAX {
        return (current, max);
    }
    // cgroup v1 fallback
    let used = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.usage_in_bytes")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    let limit = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.limit_in_bytes")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    (used, limit)
}

// Returns (swap_used, swap_max). Falls back to (0, 0) when swap is disabled.
fn read_cgroup_swap() -> (u64, u64) {
    let current = std::fs::read_to_string("/sys/fs/cgroup/memory.swap.current")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    let max = std::fs::read_to_string("/sys/fs/cgroup/memory.swap.max")
        .unwrap_or_default()
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    (current, max)
}

// Sum rx_bytes across all non-loopback interfaces from /proc/net/dev.
fn read_net_rx_bytes() -> u64 {
    let content = std::fs::read_to_string("/proc/net/dev").unwrap_or_default();
    let mut total = 0u64;
    for line in content.lines().skip(2) {
        let trimmed = line.trim();
        if trimmed.starts_with("lo:") {
            continue;
        }
        // Format: "eth0: rx_bytes rx_packets ..."
        if let Some(fields) = trimmed.splitn(2, ':').nth(1) {
            let rx = fields
                .split_whitespace()
                .next()
                .unwrap_or("0")
                .parse::<u64>()
                .unwrap_or(0);
            total += rx;
        }
    }
    total
}

// Count TCP entries whose local port matches, from /proc/net/tcp.
fn read_tcp_connection_count(port: u16) -> u32 {
    let hex_port = format!("{:04X}", port);
    let content = std::fs::read_to_string("/proc/net/tcp").unwrap_or_default();
    let mut count = 0u32;
    for line in content.lines().skip(1) {
        let mut parts = line.split_whitespace();
        if let Some(local_addr) = parts.nth(1) {
            if let Some(p) = local_addr.split(':').nth(1) {
                if p.eq_ignore_ascii_case(&hex_port) {
                    count += 1;
                }
            }
        }
    }
    count
}

impl SyntheticState {
    pub fn new(mode: DatasetMode) -> Self {
        Self {
            cpu: 0.1,
            mem: 0.1,
            swap: 0.05,
            disk: 0.1,
            throughput: 0.95,
            latency: 0.1,
            direction: 1.0,
            count: 0,
            records: vec![],
            mode,
            last_cpu_usec: read_cgroup_cpu_usec(),
            last_rx_bytes: read_net_rx_bytes(),
            last_update: Instant::now(),
        }
    }

    pub fn from_csv_dataset(&mut self) -> Result<()> {
        let file_path = Path::new("vmCloud_data.csv");
        let mut rdr = csv::Reader::from_path(file_path)?;
        let max = 1.0;
        for result in rdr.records() {
            let record = result?;
            if record.get(2).unwrap().is_empty() || record.get(3).unwrap().is_empty() {
                continue;
            }
            let features = vec![
                record
                    .get(2)
                    .unwrap()
                    .parse::<f32>()
                    .unwrap()
                    .div(100.0)
                    .clamp(0.05, max),
                record
                    .get(3)
                    .unwrap()
                    .parse::<f32>()
                    .unwrap()
                    .div(100.0)
                    .clamp(0.05, max),
            ];
            self.records.push(features);
        }
        println!("Dataset has been loaded!");
        Ok(())
    }

    fn generate_by_system_metrics(&mut self) -> Vec<f32> {
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.last_update).as_secs_f32().max(0.001);

        self.count += 1;

        // CPU: fraction of the container's single allocated core consumed since last call.
        let cpu_usec = read_cgroup_cpu_usec();
        let cpu_delta = cpu_usec.saturating_sub(self.last_cpu_usec);
        self.cpu = (cpu_delta as f32 / (elapsed_secs * 1_000_000.0)).clamp(0.0, 1.0);
        self.last_cpu_usec = cpu_usec;

        // Memory: used / limit, both from the container's cgroup.
        let (mem_used, mem_limit) = read_cgroup_memory();
        self.mem = if mem_limit > 0 && mem_limit < u64::MAX {
            (mem_used as f32 / mem_limit as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Swap: used / max from cgroup; 0.0 when swap is disabled or unlimited.
        let (swap_used, swap_max) = read_cgroup_swap();
        self.swap = if swap_max > 0 && swap_max < u64::MAX {
            (swap_used as f32 / swap_max as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Throughput: inverted rx rate so that flooding this container drives it toward 0.
        // When many containers redirect here, rx_bps rises and throughput falls → model sees busy.
        let rx_bytes = read_net_rx_bytes();
        let rx_delta = rx_bytes.saturating_sub(self.last_rx_bytes);
        let rx_bps = rx_delta as f32 / elapsed_secs;
        self.throughput = (1.0 - (rx_bps / MAX_RX_BPS)).clamp(0.0, 1.0);
        self.last_rx_bytes = rx_bytes;

        // Latency: active TCP connection count on port 8123, normalized.
        let conn_count = read_tcp_connection_count(8123);
        self.latency = (conn_count as f32 / MAX_CONNECTIONS).clamp(0.0, 1.0);

        // Disk: derived from the now-real CPU and memory (disk is not the bottleneck here).
        let alpha = 0.3;
        let beta = 0.4;
        self.disk = (alpha * self.cpu + beta * self.mem).clamp(0.0, 1.0);

        self.last_update = now;

        vec![
            self.cpu,
            self.mem,
            self.swap,
            self.disk,
            self.throughput,
            self.latency,
        ]
    }

    fn generate_by_inputs(&mut self, cpu: f32, mem: f32, lowest_state: u32) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut range = (0.01, 1.0);

        if self.count > 200 {
            let state = match lowest_state {
                0 => ((0.65, 1.0), 0.65),
                1 => ((0.35, 0.7), 0.35),
                2 => ((0.01, 0.35), 0.01),
                _ => ((0.01, 1.0), 0.0),
            };
            range = state.0;
            if state.1 > 0.0 {
                self.cpu = state.1;
                self.mem = state.1;
            }
            self.count = 0;
        }

        if rng.gen_bool(0.5) {
            self.direction *= -1.0;
        }

        let record: Vec<f32> = self.records[self.count as usize].clone();

        self.cpu = record[0];

        self.mem = record[1];

        self.swap = f32::max(0.0, self.mem + rng.gen_range(0.0..0.2) - MEMORY_THRESHOLD);

        let alpha = 0.3;
        let beta = 0.4;
        self.disk =
            (alpha * self.cpu + beta * self.mem + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        let gamma = 0.5;
        self.throughput = (1.0 - (self.cpu + self.mem) / 2.0).clamp(0.0, 1.0);
        self.throughput = (self.throughput / 1.0) * (1.0 - gamma * f32::max(self.cpu, self.mem));

        let delta = 0.5;
        self.latency = (self.cpu * 0.6 + self.mem * 0.5).clamp(0.0, 1.0);
        self.latency = f32::min(
            1.0,
            (self.latency / 1.0) * (1.0 + delta * f32::max(self.cpu, self.mem)),
        );

        if (self.cpu == range.0 && self.mem == range.0)
            || (self.cpu == range.1 && self.mem == range.1)
        {
            self.direction *= -1.0;
        }

        self.count += 1;

        vec![
            self.cpu,
            self.mem,
            self.swap,
            self.disk,
            self.throughput,
            self.latency,
        ]
    }

    fn generate_by_random(&mut self, lowest_state: u32) -> Vec<f32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut range = (0.01, 1.0);

        if self.count > 200 {
            let state = match lowest_state {
                0 => ((0.65, 1.0), 0.65),
                1 => ((0.35, 0.7), 0.35),
                2 => ((0.01, 0.35), 0.01),
                _ => ((0.01, 1.0), 0.0),
            };
            range = state.0;
            if state.1 > 0.0 {
                self.cpu = state.1;
                self.mem = state.1;
            }
            self.count = 0;
        }

        self.count += 1;

        if rng.gen_bool(0.5) {
            self.direction *= -1.0;
        }

        self.cpu = (self.cpu + self.direction * rng.gen_range(0.01..0.03)).clamp(range.0, range.1);

        self.mem = (self.mem + self.direction * rng.gen_range(0.005..0.01)).clamp(range.0, range.1);

        self.swap = f32::max(0.0, self.mem + rng.gen_range(0.0..0.2) - MEMORY_THRESHOLD);

        let alpha = 0.3;
        let beta = 0.4;
        self.disk =
            (alpha * self.cpu + beta * self.mem + rng.gen_range(-0.05..0.05)).clamp(0.0, 1.0);

        let gamma = 0.5;
        self.throughput = (1.0 - (self.cpu + self.mem) / 2.0).clamp(0.0, 1.0);
        self.throughput = (self.throughput / 1.0) * (1.0 - gamma * f32::max(self.cpu, self.mem));

        let delta = 0.5;
        self.latency = (self.cpu * 0.6 + self.mem * 0.5).clamp(0.0, 1.0);
        self.latency = f32::min(
            1.0,
            (self.latency / 1.0) * (1.0 + delta * f32::max(self.cpu, self.mem)),
        );

        if (self.cpu == range.0 && self.mem == range.0)
            || (self.cpu == range.1 && self.mem == range.1)
        {
            self.direction *= -1.0;
        }

        vec![
            self.cpu,
            self.mem,
            self.swap,
            self.disk,
            self.throughput,
            self.latency,
        ]
    }

    pub fn next(&mut self, cpu: f32, mem: f32, lowest_state: u32) -> Vec<f32> {
        match self.mode {
            DatasetMode::Generative => self.generate_by_random(lowest_state),
            DatasetMode::Inputs => self.generate_by_inputs(cpu, mem, lowest_state),
            DatasetMode::SystemMetrics => self.generate_by_system_metrics(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_state_new() {
        let state = SyntheticState::new(DatasetMode::Generative);
        assert_eq!(state.cpu, 0.1);
        assert_eq!(state.mem, 0.1);
        assert_eq!(state.swap, 0.05);
        assert_eq!(state.disk, 0.1);
        assert_eq!(state.throughput, 0.95);
        assert_eq!(state.latency, 0.1);
        assert_eq!(state.direction, 1.0);
        assert_eq!(state.count, 0);
        assert!(state.records.is_empty());
        assert_eq!(state.mode, DatasetMode::Generative);
    }

    #[test]
    fn test_next_generative() {
        let mut state = SyntheticState::new(DatasetMode::Generative);
        let features = state.next(0.0, 0.0, 0);
        assert_eq!(features.len(), 6);
        for &f in &features {
            assert!((0.0..=1.0).contains(&f));
        }
        assert_eq!(state.count, 1);
    }

    #[test]
    fn test_next_generative_multiple() {
        let mut state = SyntheticState::new(DatasetMode::Generative);
        for _ in 0..10 {
            let features = state.next(0.0, 0.0, 0);
            assert_eq!(features.len(), 6);
            for &f in &features {
                assert!((0.0..=1.0).contains(&f));
            }
        }
        assert_eq!(state.count, 10);
    }
}
