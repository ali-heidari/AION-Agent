use axum::{routing::get, Router, response::Html};
use std::env;

async fn serve_dashboard() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html>
<head>
    <title>Aion Metrics Dashboard</title>
    <meta http-equiv="refresh" content="2">
    <style>
        body { font-family: monospace; margin: 20px; background: #f5f5f5; }
        table { border-collapse: collapse; width: 100%; background: white; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background: #333; color: white; }
        tr:nth-child(even) { background: #f9f9f9; }
    </style>
</head>
<body>
    <h1>Aion Agent Metrics</h1>
    <table id="metrics-table">
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td colspan="2" style="text-align: center;">Loading...</td></tr>
    </table>
    <script>
        async function loadMetrics() {
            try {
                const resp = await fetch('/data');
                const text = await resp.text();
                const rows = [];
                text.split('\n').forEach(line => {
                    if (line && !line.startsWith('#')) {
                        const [metric, value] = line.split(' ');
                        if (metric) rows.push([metric, value || 'N/A']);
                    }
                });
                const table = document.getElementById('metrics-table');
                table.innerHTML = '<tr><th>Metric</th><th>Value</th></tr>';
                rows.forEach(([m, v]) => {
                    table.innerHTML += `<tr><td>${m}</td><td>${v}</td></tr>`;
                });
            } catch (e) {
                const table = document.getElementById('metrics-table');
                table.innerHTML = '<tr><th>Metric</th><th>Value</th></tr><tr><td colspan="2">Error loading metrics</td></tr>';
            }
        }
        loadMetrics();
    </script>
</body>
</html>"#)
}

async fn proxy_metrics(host: &'static str) -> String {
    let url = format!("http://{}:9090/metrics", host);
    match reqwest::get(&url).await {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => String::new(),
    }
}

#[tokio::main]
async fn main() {
    let agent_host = env::args()
        .find(|arg| arg.starts_with("--agent-host="))
        .and_then(|arg| arg.strip_prefix("--agent-host=").map(|s| s.to_string()))
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let agent_host: &'static str = Box::leak(agent_host.into_boxed_str());

    let app = Router::new()
        .route("/", get(serve_dashboard))
        .route("/data", get(move || proxy_metrics(agent_host)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    println!("Dashboard listening on :8080");
    axum::serve(listener, app).await.unwrap();
}
