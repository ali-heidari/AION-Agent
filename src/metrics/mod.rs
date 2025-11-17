pub use nix::sys::statvfs::statvfs;
use std::net::TcpStream;
use std::time::Instant;
use std::{fs, thread, time::Duration};

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    // CPU
    pub cpu_usage_percent: f64, // 0.0 → 100.0

    // Memory
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub memory_usage_percent: f64,

    // Swap
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_usage_percent: f64,

    // Disk
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub disk_usage_percent: f64,

    // Throughput (network I/O)
    pub net_rx_bytes_per_sec: u64, // incoming
    pub net_tx_bytes_per_sec: u64, // outgoing

    // Latency (round trip)
    pub latency_ms: f64,
}

impl SystemMetrics {
    pub fn empty() -> Self {
        Self {
            cpu_usage_percent: 0.0,

            memory_used_bytes: 0,
            memory_total_bytes: 0,
            memory_usage_percent: 0.0,

            swap_used_bytes: 0,
            swap_total_bytes: 0,
            swap_usage_percent: 0.0,

            disk_used_bytes: 0,
            disk_total_bytes: 0,
            disk_usage_percent: 0.0,

            net_rx_bytes_per_sec: 0,
            net_tx_bytes_per_sec: 0,

            latency_ms: 0.0,
        }
    }
}

fn read_title(path: &str, title: &str) -> Option<u64> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut lines = content.trim().lines();
    let entry: &str;
    for line in lines {
        if line.contains(title) {
            let mut splitted = line.split_whitespace();
            splitted.next();
            entry = splitted.next()?;
            let result = entry.trim().parse().ok()?;
            return Some(result);
        }
    }
    None
}

fn read_u64(path: &str) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn cpu_usage_percent() -> Option<f64> {
    let before = read_title("/sys/fs/cgroup/cpu.stat", "usage_usec")?;
    thread::sleep(Duration::from_millis(100));
    let after = read_title("/sys/fs/cgroup/cpu.stat", "usage_usec")?;

    let delta = (after - before) as f64;
    let usage_percent = (delta / 100_000_000.0) * 100.0; // 100ms window
    Some(usage_percent)
}

pub fn get_memory_total() -> u64 {
    if let Ok(v) = fs::read_to_string("/sys/fs/cgroup/memory.max")
        && v.trim() != "max"
    {
        return v.trim().parse().unwrap_or(0);
    }

    extract_kb_from_proc("MemTotal") * 1024
}

pub fn memory_usage_bytes() -> Option<u64> {
    if let Some(v2) = read_u64("/sys/fs/cgroup/memory.current") {
        return Some(v2);
    }
    read_u64("/sys/fs/cgroup/memory/memory.usage_in_bytes")
}
fn extract_kb(s: &str) -> u64 {
    s.split_whitespace().nth(1).unwrap().parse().unwrap()
}
fn extract_kb_from_proc(key: &str) -> u64 {
    let content = fs::read_to_string("/proc/meminfo").unwrap();
    for line in content.lines() {
        if line.starts_with(key) {
            return extract_kb(line);
        }
    }
    0
}

pub fn get_swap_total() -> u64 {
    extract_kb_from_proc("SwapTotal") * 1024
}

pub fn swap_usage_bytes() -> Option<u64> {
    if let Some(v2) = read_u64("/sys/fs/cgroup/memory.swap.current") {
        return Some(v2);
    }
    read_u64("/sys/fs/cgroup/memory/memory.memsw.usage_in_bytes")
}

pub fn disk_usage_bytes(path: &str) -> Option<(u64, u64, u64)> {
    let s = statvfs(path).ok()?;

    let total = s.blocks() * s.block_size();
    let free = s.blocks_available() * s.block_size();
    let used = total - free;

    Some((total, used, free))
}

fn read_net_bytes() -> (u64, u64) {
    let content = fs::read_to_string("/proc/net/dev").unwrap();

    let mut rx = 0;
    let mut tx = 0;

    for line in content.lines().skip(2) {
        if let Some(idx) = line.find(':') {
            let parts: Vec<&str> = line[idx + 1..].split_whitespace().collect();
            rx += parts[0].parse::<u64>().unwrap_or(0);
            tx += parts[8].parse::<u64>().unwrap_or(0);
        }
    }

    (rx, tx)
}

pub fn get_net_throughput() -> (u64, u64) {
    let (rx1, tx1) = read_net_bytes();
    let now = Instant::now();
    thread::sleep(Duration::from_secs(1));
    let (rx2, tx2) = read_net_bytes();
    let elapsed = now.elapsed().as_secs_f64();

    (
        ((rx2 - rx1) as f64 / elapsed) as u64,
        ((tx2 - tx1) as f64 / elapsed) as u64,
    )
}

// ============================
//   Latency (Ping localhost)
// ============================

pub fn get_latency_ms() -> f64 {
    let start = Instant::now();
    // simplest possible: open TCP socket to local DNS
    std::net::TcpStream::connect(("127.0.0.1", 53)).ok();
    start.elapsed().as_secs_f64() * 1000.0
}
