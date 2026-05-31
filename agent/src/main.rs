mod configurations;
mod get_mac_from_arp;
mod metrics;

use aion_transporter::multicast;
use aixker_rlt::CONFIG;
use aixker_rlt::node::Node;
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
use aixker_rlt::configurations::Configurations;
use crate::get_mac_from_arp::get_mac_from_arp;
use crate::metrics::{DatasetMode, SyntheticState};

static CACHE: LazyLock<RwLock<HashMap<String, Agent>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static NOTIFY: LazyLock<tokio::sync::Notify> = LazyLock::new(tokio::sync::Notify::new);

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
        let mut state_changed = false;
        {
            let mut cache = CACHE.write().unwrap();

            let agent_option = cache.get_mut(identifier);
            agent = if let Some(existing_agent) = agent_option {
                state_changed = existing_agent.state != state;
                existing_agent.clone()
            } else {
                Agent::new(identifier, mac, state)
            };

            if update_time - agent.update_time > 10 {
                cache.remove(identifier);
                return agent;
            }

            agent.update_time = update_time;
            agent.state = state;

            if agent.mac == [0u8; 6] {
                agent.mac = get_mac_from_arp(agent.identifier.parse().unwrap())
                    .expect(
                        format!("Failed to resolve MAC address for {}", agent.identifier).as_str(),
                    )
                    .0;
            }

            cache.insert(identifier.to_owned(), agent.clone());
        }

        if state_changed {
            NOTIFY.notify_one();
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

pub fn represent(features: &[f32], action: u8, counter: u32) -> (f32, bool) {
    let mut me = ME.lock().unwrap();
    if me.state != action {
        me.state = action;
        tokio::spawn(send_hello(action, false));
        if action == 0 {
            NOTIFY.notify_one(); // wake immediately when becoming busy; healthy transition lets in-flight connections drain
        }
    }

    (1.0, true)
}

async fn on_data_received(address: SocketAddr, data: &[u8]) {
    let message = String::from_utf8(data.to_vec()).unwrap();
    println!("Received from {}: {}", address, message);
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
}

async fn listen_to_agents() {
    tokio::spawn(async {
        loop {
            if let Err(e) = aion_transporter::multicast::listen(on_data_received).await {
                println!("Multicast listener error: {:?}, restarting...", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
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

    if let Err(error) = multicast::send(message.as_str()).await {}
}
async fn send_off_machine(ip: &str) {
    let message = ip.to_string() + "=-1";
    if let Err(error) = multicast::send(message.as_str()).await {
    }
}

fn start_cache_reaper() {
    tokio::spawn(async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let evicted = {
                let mut cache = CACHE.write().unwrap();
                let before = cache.len();
                cache.retain(|_, agent| now.saturating_sub(agent.update_time) <= 10);
                before - cache.len()
            };
            if evicted > 0 {
                NOTIFY.notify_one();
            }
        }
    });
}

fn start_ai() {
    tokio::spawn(async {
        if let Err(e) = start_predicting().await {
            println!("Error while starting AI: {:?}", e);
        };
    });
}

async fn start_predicting() -> Result<()> {
    let dataset = SyntheticState::new(DatasetMode::SystemMetrics);

    let state = Arc::new(Mutex::new(dataset));
    let cloned_state = Arc::clone(&state);

    let agent_config = load_config().unwrap();
    aixker_rlt::initialize(Arc::new(Configurations::from(agent_config.as_ref())));

    Node::start(
        move |lowest_state| get_features(&mut cloned_state.lock().unwrap(), lowest_state),
        |x, y, c| represent(x, y as u8, c),
        CONFIG.get().unwrap().mode,
        "default",
    )
    .await;

    Ok(())
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

    xdp.attach(interface_name, aya::programs::XdpMode::Skb)
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
    let mut current_target: Option<String> = None;

    let ip = local_ip().unwrap();
    let local_ip: u32 = ip.to_string().parse::<Ipv4Addr>().unwrap().into();
    let local_mac: [u8; 6] = mac_address::get_mac_address().unwrap().unwrap().bytes();
    loop {
        let is_busy = busy || ME.lock().unwrap().state == 0;
        if is_busy {
            // Single parallel ping round covering all candidates including current target.
            // Eliminates the sequential double-ping (validate then search) that added ≤100ms
            // to failover; now worst-case is one 50ms round regardless of outcome.
            let mut agents_snapshot: Vec<Agent> = {
                let agents = CACHE.read().unwrap();
                agents
                    .iter()
                    .filter(|(k, v)| {
                        k.parse::<Ipv4Addr>().map(u32::from).unwrap_or(local_ip) != local_ip
                            && v.state == 2
                    })
                    .map(|(_, v)| v.clone())
                    .collect()
            };
            rand::seq::SliceRandom::shuffle(
                agents_snapshot.as_mut_slice(),
                &mut rand::thread_rng(),
            );

            let mut join_set = tokio::task::JoinSet::new();
            for agent in &agents_snapshot {
                let id = agent.identifier.clone();
                join_set.spawn(async move {
                    let out = tokio::process::Command::new("ping")
                        .args(["-c", "1", "-W", "0.1", &id])
                        .output()
                        .await
                        .expect("ping failed");
                    let alive = !String::from_utf8_lossy(&out.stdout)
                        .contains("100% packet loss");
                    (id, alive)
                });
            }

            let mut ping_results: HashMap<String, bool> = HashMap::new();
            while let Some(res) = join_set.join_next().await {
                let (id, alive) = res.expect("ping task panicked");
                ping_results.insert(id, alive);
            }

            let current_still_valid = current_target
                .as_deref()
                .map_or(false, |t| *ping_results.get(t).unwrap_or(&false));

            if current_still_valid {
                found_agent = true;
            } else {
                current_target = None;
                let _ = server_map.remove(&0);

                for agent in &agents_snapshot {
                    if !ping_results.get(&agent.identifier).unwrap_or(&false) {
                        continue;
                    }
                    if agent.state == 2 {
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
                        for b in target_mac {
                            bytes.push(b);
                        }
                        let data: [u8; 20] = bytes.try_into().expect("Not same length!");
                        server_map
                            .insert(0, data, 0)
                            .expect("No server details defined!");
                        current_target = Some(agent.identifier.clone());
                        println!("Redirecting to agent {} at {}", agent.identifier, ipv4_addr);
                        found_agent = true;
                        break;
                    }
                }
            }
        } else {
            current_target = None;
        }

        if !found_agent && let core::result::Result::Ok(_) = server_map.remove(&0) {
                println!("No agent found, removing server from map.");
        }

        found_agent = false;

        // When busy with an active target, health-check every 1s regardless of interval_secs
        // to bound the stale-SERVERMAP window. Otherwise respect the full interval.
        let sleep_secs = if is_busy && current_target.is_some() {
            1u64
        } else {
            CONFIG.get().unwrap().interval_secs
        };
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(sleep_secs)) => {},
            _ = NOTIFY.notified() => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let startup_config = load_config().unwrap();
    let mut busy = startup_config.busy;
    let mut neighbor = startup_config.neighbor.clone();

    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        let option = &args[2];
        if option.eq("neighbor") {
            if args.len() > 3 {
                neighbor = args[3].clone();
            }
        } else if option.eq("busy") {
            busy = true;
        }
    }

    if !neighbor.is_empty() {
        add_agents((neighbor + ",2").as_str(), '|');
    }

    listen_to_agents().await;

    send_hello(2, true).await;

    start_cache_reaper();
    start_ai();

    println!("Load ebpf - XDP Program [busy={}]", busy);
    tokio::spawn(async move {
        load_ebf(busy).await.expect("Can't load the ebpf!");
    });

    std::future::pending::<()>().await;

    Ok(())
}
