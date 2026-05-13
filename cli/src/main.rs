use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aion", about = "Aion agent CLI")]
struct Cli {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show raw Prometheus metrics from the agent
    Metrics,
    /// Show agent state summary
    Status,
}

fn main() {
    let cli = Cli::parse();
    let url = format!("http://{}:9090/metrics", cli.host);

    match cli.command {
        Commands::Metrics | Commands::Status => {
            match reqwest::blocking::get(&url) {
                Ok(resp) => {
                    let text = resp.text().unwrap_or_default();
                    for line in text.lines() {
                        if line.starts_with("aion_") && !line.starts_with('#') {
                            println!("{}", line);
                        }
                    }
                }
                Err(e) => eprintln!("Could not reach agent at {}: {}", url, e),
            }
        }
    }
}
