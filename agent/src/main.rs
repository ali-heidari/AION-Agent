mod configurations;
mod mock;
mod reward;
mod get_mac_from_arp;

use aixker_rlt::CONFIG;
use aixker_rlt::node::Node;
use aion_transporter::multicast;
use aion_transporter::quic::client;
use anyhow::Context;
use anyhow::{Ok, Result};
use aya::Pod;
use local_ip_address::local_ip;
use std::collections::HashMap;
use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::LazyLock;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};

use crate::configurations::load_config;
use crate::mock::{DatasetMode, SyntheticState};
use crate::reward::compute_reward_with_success;

static CACHE: LazyLock<RwLock<HashMap<String, Agent>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static ME: LazyLock<Mutex<Agent>> = LazyLock::new(|| {
    let local_ip = local_ip().unwrap();
    let agent = Agent::new(local_ip.to_string().as_str(), [0u8; 6], 2);
    Mutex::new(agent)
});

#[derive(Clone)]
struct Agent {
    identifier: String,
    mac: [u8; 6],
    state: u8,
    update_time: u64,
}

impl Agent {
    fn new(identifier: &str, mac: [u8; 6], state: u8) -> Self {
        Agent {
            identifier: identifier.to_owned(),
            state,
            mac,
            update_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    fn parse(data: &str) -> Agent {
        let segments: Vec<&str> = data.split(",").collect();
        let identifier: &str = segments[0];
        let state: u8 = segments[1].parse().unwrap();

        let mut mac: [u8; 6] = [0u8; 6];
        if segments.len() > 2 {
            let mac_temp: Vec<u8> = segments[2]
                .split(":")
                .map(|x| u8::from_str_radix(x, 16).expect("Invalid hex digit"))
                .collect();
            mac = mac_temp.try_into().expect("Invalid MAC address length");
        }
        let update_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut agent;
        {
            let mut cache = CACHE.write().unwrap();

            let agent_option = cache.get_mut(identifier);
            agent = if let Some(existing_agent) = agent_option {
                existing_agent.clone()
            } else {
                Agent::new(identifier, mac, state)
            };

            if agent.update_time - update_time > 10 {
                cache.remove(identifier);
                return agent;
            }

            agent.update_time = update_time;
            cache.insert(identifier.to_owned(), agent.clone());
        }

        agent
    }

    fn stringify(&self) -> String {
        self.identifier.to_string()
            + ","
            + self.mac.map(|x| x.to_string()).join(":").as_str()
            + ","
            + self.state.to_string().as_str()
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
    if message.contains('=') {
        {
            let mut cache = CACHE.write().unwrap();
            cache.remove(message.split('=').next().unwrap());
            return;
        }
    }
    if message.contains("[over]") {
        return;
    }

    add_agents(&message, '|');

    // represent(2).await;
}

pub async fn represent(action: u8) {
    send_hello(action, false).await;
    return;
    ME.lock().unwrap().state = action;
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
            if ip.eq(local_ip().unwrap().to_string().as_str()) {
                continue;
            }
            println!("represent to: {}", ip);
            if let Err(e) = client::send(ip, 4433, message.as_bytes()).await {
                {
                    let mut cache = CACHE.write().unwrap();
                    cache.remove(ip);
                }
                println!("Error while sending IP(s) to agents: {:?}", e);
                println!("Removing agent {}", ip);
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

async fn send_hello(action: u8, include_mac: bool) {
    let local_ip = local_ip().unwrap();
    let mut message = local_ip.to_string() + "," + action.to_string().as_str();

    if include_mac {
        message = message
            + ","
            + mac_address::get_mac_address()
                .unwrap()
                .unwrap()
                .to_string()
                .as_str()
    }

    if let Err(error) = multicast::send(message.as_str()).await {
        println!("Error while sending hello: {:?}", error);
    }
}
async fn send_off_machine(ip: &str) {
    let local_ip = local_ip().unwrap();
    let message = local_ip.to_string() + "=-1";
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

    aixker_rlt::initialize(load_config().unwrap());

    Node::start(
        move |lowest_state| get_features(&mut cloned_state.lock().unwrap(), lowest_state),
        |x, y,c| compute_reward_with_success(x, y as u8),
        CONFIG.get().unwrap().mode,
        "model-128.json"
    )
    .await;

    Ok(())
}

fn on_message_received(ip: String, message: String) {
    println!("Quic message received-> [{}]: {}", ip, message);

    let details = ip + "," + &message;

    add_agents(details.as_str(), '|');
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RedirectionData {
    local_ip: u32,
    local_mac: [u8; 6],
    server_ip: u32,
    server_mac: [u8; 6],
}

unsafe impl Pod for RedirectionData {}

async fn load_ebf(busy: bool) -> core::result::Result<(), anyhow::Error> {
    let mut bpf = aya::Ebpf::load(aya::include_bytes_aligned!("./libebpf.so"))
        .context("Failed to load eBPF object")?;

    let default_interface_output = std::process::Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .expect("Can't find default network interface!")
        .stdout;

    let text = String::from_utf8_lossy(&default_interface_output);
    let interface_name = text.split(' ').collect::<Vec<&str>>()[4];

    match aya_log::EbpfLogger::init(&mut bpf) {
        Err(e) => {
            // This can happen if you remove all log statements from your eBPF program.
            println!("failed to initialize eBPF logger: {e}");
        }
        core::result::Result::Ok(logger) => {
            let mut logger =
                tokio::io::unix::AsyncFd::with_interest(logger, tokio::io::Interest::READABLE)?;
            tokio::task::spawn(async move {
                loop {
                    let mut guard = logger.readable_mut().await.unwrap();
                    guard.get_inner_mut().flush();
                    guard.clear_ready();
                }
            });
        }
    }

    let program = bpf
        .program_mut("simple_xdp")
        .ok_or_else(|| anyhow::anyhow!("Program 'simple_xdp' not found"))?;

    let xdp: &mut aya::programs::Xdp = program.try_into().context("Failed to cast to Xdp")?;

    xdp.load().context("Failed to load XDP program")?;

    xdp.attach(interface_name, aya::programs::XdpMode::default())
        .context("Failed to attach XDP to default interface!")?;

    println!(
        "XDP ATTACHED to {}. Send ping → watch dmesg",
        interface_name
    );

    let mut devmap: aya::maps::DevMap<_> = bpf.map_mut("DEVMAP").unwrap().try_into()?;
    let ifindex = nix::net::if_::if_nametoindex(interface_name)?;
    println!("Redirecting to ifindex {}", ifindex);
    devmap.set(0, ifindex, None, 0)?;

    let mut blocklist: aya::maps::HashMap<_, u32, u32> =
        aya::maps::HashMap::try_from(bpf.map_mut("BLOCKLIST").unwrap())?;
    let block_addr: u32 = std::net::Ipv4Addr::new(192, 168, 100, 100).into();
    blocklist.insert(block_addr, 0, 0)?;

    let mut server_map: aya::maps::HashMap<_, u32, [u8; 20]> =
        aya::maps::HashMap::try_from(bpf.map_mut("SERVERMAP").unwrap()).unwrap();
    let mut found_agent: bool = false;

    let ip = local_ip().unwrap();
    let local_ip: u32 = ip.to_string().parse::<Ipv4Addr>().unwrap().into();
    let local_mac: [u8; 6] = mac_address::get_mac_address().unwrap().unwrap().bytes();
    loop {
        {
            let agents = CACHE.read().unwrap();

            if busy || ME.lock().unwrap().state == 0 {
                for agent_borrowed in agents.iter() {
                    let agent = agent_borrowed.1.clone();

                    let default_interface_output = std::process::Command::new("ping")
                        .args(["-c", "1", "-W", "1", agent.identifier.as_str()])
                        .output()
                        .expect("Can't find default network interface!")
                        .stdout;
                    let text: std::borrow::Cow<'_, str> =
                        String::from_utf8_lossy(&default_interface_output);

                    if text.contains("100% packet loss") {
                        send_off_machine(&agent.identifier);
                        continue;
                    }

                    if agent.state == 2 || agent.state == 1 {
                        println!(">>>>>>>>> {}", agent.stringify());
                        let ipv4_addr: Ipv4Addr = agent.identifier.parse().unwrap();

                        let ip_as_u32: u32 = ipv4_addr.into();
                        if local_ip == ip_as_u32 {
                            continue;
                        }
                        let mut bytes: Vec<u8> = local_ip.to_be_bytes().into();
                        for b in local_mac {
                            bytes.push(b);
                        }
                        for b in ip_as_u32.to_be_bytes() {
                            bytes.push(b);
                        }
                        let target_mac: [u8; 6] = agent.mac;
                        // if let Some(val) = get_mac_from_arp(ipv4_addr) {
                        //     val.0
                        // } else {
                        //     println!("Failed to parse target MAC: {:?}", ipv4_addr);
                        //     continue;
                        // };
                        for b in target_mac {
                            bytes.push(b);
                        }
                        let data: [u8; 20] = bytes.try_into().expect("Not same length!");
                        server_map
                            .insert(0, data, 0)
                            .expect("No server details defined!");
                        println!(
                            "Agent: {}({}) State: {}",
                            agent.identifier, ip_as_u32, agent.state
                        );
                        found_agent = true;
                        break;
                    }
                }
            }

            if !found_agent && let core::result::Result::Ok(_) = server_map.remove(&0) {
                println!("No suitable agent found, clearing server map");
            }

            found_agent = false;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(
            CONFIG.get().unwrap().interval_secs,
        ))
        .await;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Let's begin...");
    env_logger::init();
    println!("Log initialized...!");

    let mut busy = false;

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        let option = &args[2];
        println!("The args: {:?}", &args);
        if option.eq("neighbor") {
            let neighbor = &args[2];
            add_agents((neighbor.to_string() + ",2").as_str(), '|');
        } else if option.eq("busy") {
            busy = true;
        }
    }
    println!("Quic initializing...");

    aion_transporter::quic::init();

    println!("Broadcasting hello!");
    send_hello(2, true).await;
    println!("Listening to multicast packets!");
    listen_to_agents().await;
    println!("Start AI");
    start_ai();
    // println!("Running quic server!");
    // task::spawn(async {
    //     if let Err(e) = start_quic(4433, on_message_received).await {
    //         println!("Error while starting QUIC server: {:?}", e);
    //     }
    // });
    println!("Load ebpf - XDP Program [busy={}]", busy);

    tokio::spawn(async move {
        load_ebf(busy).await.expect("Can't load the ebpf!");
    });

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = std::future::pending::<()>() => {},
    }
    println!("Exiting...");
    Ok(())
}
