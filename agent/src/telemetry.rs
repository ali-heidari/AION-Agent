use prometheus::{
    register_gauge, register_int_gauge, Encoder, Gauge, IntGauge, TextEncoder,
};
use std::sync::LazyLock;

pub static AGENT_STATE: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("aion_agent_state", "Current agent load state (0=free,1=mid,2=busy)")
        .unwrap()
});
pub static PEER_COUNT: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("aion_peer_count", "Number of known peer agents").unwrap()
});
pub static CPU_USAGE: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!("aion_cpu_usage_percent", "Host CPU usage percent").unwrap()
});
pub static MEM_USAGE: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!("aion_memory_usage_percent", "Host memory usage percent").unwrap()
});
pub static NET_RX: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("aion_net_rx_bytes_per_sec", "Network RX bytes/sec").unwrap()
});
pub static NET_TX: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!("aion_net_tx_bytes_per_sec", "Network TX bytes/sec").unwrap()
});

pub fn metrics_text() -> String {
    let encoder = TextEncoder::new();
    let mf = prometheus::gather();
    let mut buf = Vec::new();
    encoder.encode(&mf, &mut buf).unwrap();
    String::from_utf8(buf).unwrap()
}
