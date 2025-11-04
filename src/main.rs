mod configurations;
mod mock;
mod reward;

use aion_rlt::CONFIG;
use aion_rlt::node::Node;
use aion_transporter::multicast;
use aion_transporter::quic::client;
use aion_transporter::quic::server::start_quic;
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

        let agent = Agent::new(identifier, state);
        {
            let mut cache = CACHE.write().unwrap();
            cache.insert(identifier.to_owned(), agent.clone());
        }

        agent
    }

    fn stringify(&self) -> String {
        self.identifier.to_string() + "," + self.state.to_string().as_str()
    }
}

fn get_features(state: &mut SyntheticState, lowest_state: u32) -> Vec<f32> {
    state.next(0.0, 0.0, lowest_state)
}

fn add_agents(details: &str, separator: char) {
    let agents = details.split(separator);

    for agent in agents {
        Agent::parse(agent);
    }
}

async fn on_data_received(address: SocketAddr, data: &[u8]) {
    let message = String::from_utf8(data.to_vec()).unwrap();
    println!("message came from {} says: {:?}", address.ip(), message);
    if message.contains("[over]") {
        return;
    }

    add_agents(&message, '|');

    represent(2).await;
}

pub async fn represent(action: u8) {
    let all_ips: Vec<String>;
    {
        let cache = CACHE.read().unwrap();
        all_ips = cache
            .iter()
            .map(|entry| entry.1.stringify())
            .collect::<Vec<String>>();
    }

    let message = action.to_string() + "|" + all_ips.join("|").as_str();
    if !all_ips.is_empty() {
        for ip in all_ips
            .iter()
            .map(|ip| *ip.split(",").collect::<Vec<&str>>().first().unwrap())
        {
            println!("represent to: {}", ip);
            if let Err(e) = client::send(ip, 4433, message.as_bytes()).await {
                {
                    let mut cache = CACHE.write().unwrap();
                    cache.remove(ip);
                }
                println!("Error while sending IP(s) to agents: {:?}", e);
            }
        }
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

fn start_ai() {
    tokio::spawn(async {
        if let Err(e) = start_predicting().await {
            println!("Error while starting AI: {:?}", e);
        };
        println!("Starting AI thread closed");
    });
}

async fn start_predicting() -> Result<()> {
    let dataset = SyntheticState::new(DatasetMode::SystemMetrics);

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

fn on_message_received(ip: String, message: String) {
    println!("Quic message received-> [{}]: {}", ip, message);

    let details = ip + "," + &message;

    add_agents(details.as_str(), '|');
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        let option = &args[1];
        if option.eq("neighbor") {
            let neighbor = &args[2];
            add_agents((neighbor.to_string() + ",2").as_str(), '|');
        }
    }

    aion_transporter::quic::init();

    println!("Broadcasting hello!");
    send_hello().await;
    println!("Listening to multicast packets!");
    listen_to_agents().await;
    println!("Start AI");
    start_ai();
    println!("Running quic server!");
    if let Err(error) = start_quic(4433, on_message_received).await {
        println!("Error while starting quic server: {}", error);
    }

    loop {
        println!("looping");
        sleep(Duration::from_secs(2));
    }
}
