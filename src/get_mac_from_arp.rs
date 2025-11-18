use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct MacAddress(pub [u8; 6]);

pub fn get_mac_from_arp(ip: Ipv4Addr) -> Option<MacAddress> {
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
