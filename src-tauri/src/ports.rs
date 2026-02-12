use serde::Serialize;
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

fn get_process_for_inode(inode: &str) -> Option<(u32, String)> {
    let proc_dir = fs::read_dir("/proc").ok()?;
    for entry in proc_dir.flatten() {
        let pid_str = entry.file_name().to_string_lossy().to_string();
        if let Ok(pid) = pid_str.parse::<u32>() {
            let fd_dir = format!("/proc/{}/fd", pid);
            if let Ok(fds) = fs::read_dir(&fd_dir) {
                for fd in fds.flatten() {
                    if let Ok(link) = fs::read_link(fd.path()) {
                        let link_str = link.to_string_lossy();
                        if link_str.contains(&format!("socket:[{}]", inode)) {
                            let comm = fs::read_to_string(format!("/proc/{}/comm", pid))
                                .unwrap_or_default()
                                .trim()
                                .to_string();
                            return Some((pid, comm));
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn scan_ports() -> Vec<SystemPort> {
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

                let (pid, process_name) = get_process_for_inode(inode)
                    .map(|(p, n)| (Some(p), Some(n)))
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
