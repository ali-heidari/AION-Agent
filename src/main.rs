mod configurations;
mod mock;
mod reward;

use aion_rlt::CONFIG;
use aion_rlt::node::Node;
use aion_transporter::multicast;
use anyhow::{Ok, Result};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use crate::configurations::load_config;
use crate::mock::{DatasetMode, SyntheticState};
use crate::reward::compute_reward_with_success;

static CACHE: LazyLock<RwLock<HashMap<String, Agent>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Clone)]
struct Agent {
    identifier: String,
    state: u8,
}

impl Agent {
    fn new(identifier: &str, state: u8) -> Self {
        Agent {
            identifier: identifier.to_owned(),
            state,
        }
    }

    fn parse(data: &str) -> Agent {
        let segments: Vec<&str> = data.split(",").collect();
        let identifier: &str = segments[0];
        let state: u8 = segments[1].parse().unwrap();

        Agent::new(identifier, state)
    }

    fn stringify(&self) -> String {
        self.identifier.to_string() + "," + self.state.to_string().as_str()
    }
}

fn get_features(state: &mut SyntheticState, lowest_state: u32) -> Vec<f32> {
    state.next(0.0, 0.0, lowest_state)
}

async fn on_data_received(address: SocketAddr, data: &[u8]) {
    let message = String::from_utf8(data.to_vec()).unwrap();
    println!("message came from {} says: {:?}", address.ip(), message);
    if message.contains("[over]") {
        return;
    }

    {
        let agents = message.split("|");
        let mut cache = CACHE.write().unwrap();

        for agent in agents {
            println!("{}", agent);
            cache.insert(address.ip().to_string(), Agent::parse(agent));
        }
    }

    {
        let cache = CACHE.read().unwrap();
        let all_ips = cache
            .iter()
            .map(|entry| entry.1.stringify())
            .collect::<Vec<String>>()
            .join("|");

        println!("We know IP(s): {}\n***********", all_ips)
    }
}

async fn listen_to_agents() {
    tokio::spawn(async {
        if let Err(e) = aion_transporter::multicast::listen(on_data_received).await {
            println!("Error while listening for multicast packets: {:?}", e);
        };
        println!("Listening thread closed");
    });
}
use local_ip_address::local_ip;
async fn send_hello() {
    let local_ip = local_ip().unwrap();
    let message = local_ip.to_string() + ",2";
    if let Err(error) = multicast::send(message.as_str()).await {
        println!("Error while sending hello: {:?}", error);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    println!("running");

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
