use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;
use std::vec;

pub struct ARPTable {
    pub ip: String,
    hw_type: u8,
    flags: u8,
    hw_address: String,
    mask: String,
    device: String,
}

#[derive(Debug, Clone)]
pub struct MacAddress(pub [u8; 6]);

pub fn get_machines_from_arp() -> Option<Vec<Ipv4Addr>> {
    let file = File::open("/proc/net/arp").ok()?;
    let reader = BufReader::new(file);
    let mut ips: Vec<Ipv4Addr> = vec![];

    // Skip header line
    for (i, line) in reader.lines().enumerate() {
        let line = line.ok()?;
        if i <= 1 {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let entry_ip = parts[0];
        ips.push(entry_ip.parse().unwrap());
    }

    Some(ips)
}

pub fn get_mac_from_arp(ip: Ipv4Addr) -> Option<MacAddress> {
    std::process::Command::new("ping")
        .args(["-c", "1", "-W", "0.2", ip.to_string().as_str()])
        .output()
        .expect("Can't find default network interface!");

    let file = File::open("/proc/net/arp").ok()?;
    let reader = BufReader::new(file);
    let ip_str = ip.to_string();

    // Skip header line
    for (i, line) in reader.lines().enumerate() {
        let line = line.ok()?;
        if i == 0 {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }

        let entry_ip = parts[0];
        if entry_ip == ip_str {
            let mac_str = parts[3];

            // Parse MAC: "aa:bb:cc:dd:ee:ff"
            let mut mac_bytes = [0u8; 6];
            for (i, part) in mac_str.split(':').enumerate() {
                if let Ok(byte) = u8::from_str_radix(part, 16) {
                    mac_bytes[i] = byte;
                } else {
                    return None;
                }
            }

            return Some(MacAddress(mac_bytes));
        }
    }

    None
}

pub fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut lines = vec![];
    for line in reader.lines() {
        let line = line?;
        lines.push(line);
    }

    Ok(lines)
}
