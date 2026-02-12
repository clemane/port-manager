use serde::Serialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Clone)]
pub struct SystemPort {
    pub protocol: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub state: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

fn parse_hex_port(hex: &str) -> u16 {
    u16::from_str_radix(hex, 16).unwrap_or(0)
}

fn tcp_state(state: &str) -> &'static str {
    match state {
        "0A" => "LISTEN",
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0B" => "CLOSING",
        _ => "UNKNOWN",
    }
}

/// Build a mapping of socket inode -> (pid, process_name) by scanning /proc once.
/// This is O(total_fds) instead of O(connections * total_fds).
fn build_inode_map() -> HashMap<String, (u32, String)> {
    let mut map = HashMap::new();
    let proc_dir = match fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return map,
    };

    for entry in proc_dir.flatten() {
        let pid_str = entry.file_name().to_string_lossy().to_string();
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let fd_dir = format!("/proc/{}/fd", pid);
        let fds = match fs::read_dir(&fd_dir) {
            Ok(f) => f,
            Err(_) => continue,
        };

        // Read comm lazily (only if we find a socket)
        let mut comm: Option<String> = None;

        for fd in fds.flatten() {
            if let Ok(link) = fs::read_link(fd.path()) {
                let link_str = link.to_string_lossy();
                if let Some(start) = link_str.find("socket:[") {
                    let inode_start = start + 8;
                    if let Some(end) = link_str[inode_start..].find(']') {
                        let inode = &link_str[inode_start..inode_start + end];
                        let name = comm.get_or_insert_with(|| {
                            fs::read_to_string(format!("/proc/{}/comm", pid))
                                .unwrap_or_default()
                                .trim()
                                .to_string()
                        });
                        map.insert(inode.to_string(), (pid, name.clone()));
                    }
                }
            }
        }
    }
    map
}

pub fn scan_ports() -> Vec<SystemPort> {
    let inode_map = build_inode_map();
    let mut ports = Vec::new();

    for (file, proto) in [("/proc/net/tcp", "tcp"), ("/proc/net/tcp6", "tcp6")] {
        if let Ok(content) = fs::read_to_string(file) {
            for line in content.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() < 10 {
                    continue;
                }

                let local_parts: Vec<&str> = fields[1].split(':').collect();
                let remote_parts: Vec<&str> = fields[2].split(':').collect();
                if local_parts.len() < 2 || remote_parts.len() < 2 {
                    continue;
                }

                let local_port = parse_hex_port(local_parts.last().unwrap_or(&"0"));
                let remote_port = parse_hex_port(remote_parts.last().unwrap_or(&"0"));
                let state = tcp_state(fields[3]).to_string();
                let inode = fields[9];

                let (pid, process_name) = inode_map
                    .get(inode)
                    .map(|(p, n)| (Some(*p), Some(n.clone())))
                    .unwrap_or((None, None));

                ports.push(SystemPort {
                    protocol: proto.to_string(),
                    local_port,
                    remote_port,
                    state,
                    pid,
                    process_name,
                });
            }
        }
    }
    ports.sort_by_key(|p| p.local_port);
    ports.dedup_by(|a, b| {
        a.local_port == b.local_port && a.pid == b.pid && a.protocol == b.protocol
    });
    ports
}
