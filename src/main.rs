mod configurations;
mod mock;
mod reward;

use aion_rlt::CONFIG;
use aion_rlt::node::Node;
use aion_transporter;
use anyhow::{Ok, Result};
use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::u32;

use crate::configurations::load_config;
use crate::mock::{DatasetMode, SyntheticState};
use crate::reward::compute_reward_with_success;

static CACHE: LazyLock<RwLock<HashMap<String, Agent>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Clone)]
struct Agent {
    identifier: String,
    update_time: u64,
    state: u8,
}

impl Agent {
    fn new(identifier: &str, update_time: u64, state: u8) -> Self {
        Agent {
            identifier: identifier.to_owned(),
            update_time,
            state,
        }
    }

    fn parse(data: &str) -> Agent {
        let segments: Vec<&str> = data.split(",").collect();
        let identifier: &str = segments[0];
        let update_time: u64 = segments[1].parse().unwrap();
        let state: u8 = segments[2].parse().unwrap();

        Agent::new(identifier, update_time, state)
    }

    fn stringify(&self) -> String {
        self.update_time.to_string() + "," + self.state.to_string().as_str()
    }
}

fn get_features(state: &mut SyntheticState, lowest_state: u32) -> Vec<f32> {
    let features = state.next(0.0, 0.0, lowest_state);
    features
}

async fn on_data_received(address: SocketAddr, data: &[u8]) {
    let message = String::from_utf8(data.to_vec()).unwrap();
    if message.contains("[over]") {
        return;
    }

    {
        let agents = message.split("|");
        let mut cache = CACHE.write().unwrap();

        println!("Received agents: {:?}", agents);
        for agent in agents {
            println!("{}",agent);
            cache.insert(address.ip().to_string(), Agent::parse(agent));
        }
    }

    let mut reply_message = String::new();
    {
        let cache = CACHE.read().unwrap();
        let all_ips = cache
            .iter()
            .map(|entry| entry.1.stringify())
            .collect::<Vec<String>>()
            .join("|");

        reply_message = all_ips + "[over]";
    }
    aion_transporter::multicast::send(&reply_message).await;
    println!("Sent reply: {}", reply_message);
}

async fn listen_to_agents() {
    tokio::spawn(async {
        aion_transporter::multicast::listen(on_data_received).await;
    });
}
use local_ip_address::local_ip;
async fn send_hello() {
    let local_ip = local_ip().unwrap();
    let message = local_ip.to_string() + ",1221,2";
    aion_transporter::multicast::send(message.as_str()).await;
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    send_hello().await;
    listen_to_agents().await;

    loop {
        sleep(Duration::from_secs(2));
    }

    return Ok(());

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Ok(());
    }
    let mode = &args[1];

    let mode = match mode.as_str() {
        "metrics" => DatasetMode::SystemMetrics,
        "generative" => DatasetMode::Generative,
        "csv" => DatasetMode::Inputs,
        _ => DatasetMode::Generative,
    };

    let mut dataset = SyntheticState::new(mode.clone());
    if mode == DatasetMode::Inputs {
        dataset.from_csv_dataset().ok();
    }
    let state = Arc::new(Mutex::new(dataset));
    let cloned_state = Arc::clone(&state);

    aion_rlt::initialize(load_config().unwrap());

    Node::start(
        move |lowest_state| get_features(&mut cloned_state.lock().unwrap(), lowest_state),
        |x, y| compute_reward_with_success(x, y as u8),
        CONFIG.get().unwrap().mode,
    )
    .await;

    Ok(())
}
