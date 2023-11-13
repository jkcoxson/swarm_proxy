// Jackson Coxson

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use json::JsonValue;

#[derive(Debug)]
pub struct Configs {
    pub udp: HashMap<SocketAddrV4, SocketAddrV4>,
    pub tcp: HashMap<SocketAddrV4, SocketAddrV4>,
}

#[derive(Debug)]
enum Parse {
    Udp,
    Tcp,
}

impl Configs {
    pub fn load() -> Result<Self, String> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            return Err("No arguments provided!".to_string());
        }
        if args[1].ends_with(".json") {
            // Get the JSON
            let j = std::fs::read_to_string(&args[1])
                .map_err(|e| format!("Failed to read JSON file: {:?}", e))?;
            Self::load_from_json(&j)
        } else {
            Self::load_from_args(&args[1..])
        }
    }
    pub fn load_from_args(args: &[String]) -> Result<Self, String> {
        let mut udp_map = HashMap::new();
        let mut tcp_map = HashMap::new();
        let host = Ipv4Addr::new(0, 0, 0, 0);

        let target = args[0].clone();
        let target = Ipv4Addr::from_str(&target).map_err(|_| "Invalid target IP address")?;

        let mut parse_mode = None;
        for arg in &args[1..] {
            match arg.as_str() {
                "udp" => parse_mode = Some(Parse::Udp),
                "tcp" => parse_mode = Some(Parse::Tcp),
                _ => {
                    let (host_ports, remote_ports) = parse_port_range(arg)?;
                    for i in 0..host_ports.len() {
                        match parse_mode {
                            Some(Parse::Udp) => udp_map.insert(
                                SocketAddrV4::new(host, host_ports[i]),
                                SocketAddrV4::new(target, remote_ports[i]),
                            ),
                            Some(Parse::Tcp) => tcp_map.insert(
                                SocketAddrV4::new(host, host_ports[i]),
                                SocketAddrV4::new(target, remote_ports[i]),
                            ),
                            _ => return Err("TCP or UDP not selected".to_string()),
                        };
                    }
                }
            }
        }

        Ok(Self {
            udp: udp_map,
            tcp: tcp_map,
        })
    }

    pub fn load_from_json(config_string: &str) -> Result<Self, String> {
        let mut udp = HashMap::new();
        let mut tcp = HashMap::new();
        let config = match json::parse(config_string) {
            Ok(c) => c,
            Err(e) => return Err(e.to_string()),
        };

        match config {
            json::JsonValue::Object(configs) => {
                for (remote, config) in configs.iter() {
                    // Attempt to parse key as an IPv4 addr
                    let remote = match Ipv4Addr::from_str(remote) {
                        Ok(r) => r,
                        Err(_) => return Err(format!("Invalid IPv4 address: {remote}")),
                    };

                    // Iterate over entries
                    match config {
                        json::JsonValue::Array(entries) => {
                            for entry in entries {
                                match entry {
                                    json::JsonValue::Object(entry) => {
                                        // Determine if range or one-off
                                        let mut host_ports = vec![];
                                        let mut remote_ports = vec![];
                                        if let Some(host_port) = entry.get("host_port") {
                                            let host_port = pls_json_number(host_port)?;
                                            let remote_port =
                                                pls_json_number_maybe(entry.get("remote_port"))?;
                                            host_ports.push(host_port);
                                            remote_ports.push(remote_port);
                                        } else if let Some(host_port_start) =
                                            entry.get("host_port_start")
                                        {
                                            let host_port_start = pls_json_number(host_port_start)?;
                                            let host_port_end =
                                                pls_json_number_maybe(entry.get("host_port_end"))?;
                                            let remote_port_start = pls_json_number_maybe(
                                                entry.get("remote_port_start"),
                                            )?;
                                            let remote_port_end = pls_json_number_maybe(
                                                entry.get("remote_port_end"),
                                            )?;
                                            host_ports.extend(host_port_start..=host_port_end);
                                            remote_ports
                                                .extend(remote_port_start..=remote_port_end);
                                            if host_ports.len() != remote_ports.len() {
                                                return Err(format!(
                                                    "Port length mismatch defined for entry: {:?}",
                                                    entry.dump()
                                                ));
                                            }
                                        } else {
                                            return Err(format!(
                                                "No ports defined for entry: {:?}",
                                                entry.dump()
                                            ));
                                        }
                                        let bind = match entry.get("bind") {
                                            Some(bind) => bind.as_str().unwrap_or("0.0.0.0"),
                                            None => "0.0.0.0",
                                        };
                                        let bind = match Ipv4Addr::from_str(bind) {
                                            Ok(b) => b,
                                            Err(_) => {
                                                return Err(format!(
                                                    "Bind address is invalid: {bind}",
                                                ));
                                            }
                                        };
                                        let mode = match entry.get("mode") {
                                            Some(mode) => match mode.as_str() {
                                                Some(mode) => mode,
                                                None => {
                                                    return Err(format!(
                                                        "No mode defined for entry: {:?}",
                                                        entry.dump()
                                                    ));
                                                }
                                            },
                                            None => {
                                                return Err(format!(
                                                    "No mode defined for entry: {:?}",
                                                    entry.dump()
                                                ));
                                            }
                                        };

                                        for i in 0..host_ports.len() {
                                            match mode {
                                                "udp" => udp.insert(
                                                    SocketAddrV4::new(bind, host_ports[i]),
                                                    SocketAddrV4::new(remote, remote_ports[i]),
                                                ),
                                                "tcp" => tcp.insert(
                                                    SocketAddrV4::new(bind, host_ports[i]),
                                                    SocketAddrV4::new(remote, remote_ports[i]),
                                                ),
                                                _ => {
                                                    return Err(format!("Invalid mode: {mode}"));
                                                }
                                            };
                                        }
                                    }
                                    _ => {
                                        return Err(format!(
                                            "Not an object for IPv4 address: {remote}"
                                        ))
                                    }
                                }
                            }
                        }
                        _ => return Err(format!("Expected a list of entries for {:?}", remote)),
                    }
                }
            }
            _ => return Err("Expected an object of objects".to_string()),
        }

        Ok(Self { udp, tcp })
    }
}

fn parse_port_range(arg: &str) -> Result<(Vec<u16>, Vec<u16>), &'static str> {
    if arg.contains(':') {
        let parts: Vec<&str> = arg.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid port range syntax, unexpected :");
        }

        Ok(if parts[0].contains('-') {
            let (start, end) = parse_port_ends(parts[0])?;
            let host_ports: Vec<u16> = (start..=end).collect();

            let (start, end) = parse_port_ends(parts[1])?;
            let remote_ports: Vec<u16> = (start..=end).collect();

            if host_ports.len() != remote_ports.len() {
                return Err("Port range length mismatch");
            }

            (host_ports, remote_ports)
        } else {
            (vec![parse_single(parts[0])?], vec![parse_single(parts[1])?])
        })
    } else if arg.contains('-') {
        let (start, end) = parse_port_ends(arg)?;
        let host_ports: Vec<u16> = (start..=end).collect();
        Ok((host_ports.clone(), host_ports))
    } else {
        let port = arg
            .parse::<u16>()
            .map_err(|_| "Invalid port range syntax, not an int")?;
        Ok((vec![port], vec![port]))
    }
}

fn parse_port_ends(ends: &str) -> Result<(u16, u16), &'static str> {
    let port_ends: Vec<&str> = ends.split('-').collect();
    if port_ends.len() != 2 {
        return Err("Invalid port range syntax, unexpected -");
    }

    let start = port_ends[0]
        .parse::<u16>()
        .map_err(|_| "Invalid port range syntax, not an int")?;

    let end = port_ends[1]
        .parse::<u16>()
        .map_err(|_| "Invalid port range syntax, not an int")?;

    if start > end {
        return Err("Invalid port range syntax, start > end");
    }

    Ok((start, end))
}

fn parse_single(arg: &str) -> Result<u16, &'static str> {
    arg.parse::<u16>()
        .map_err(|_| "Invalid port range syntax, not an int")
}

fn pls_json_number_maybe(number: Option<&JsonValue>) -> Result<u16, &'static str> {
    match number {
        Some(n) => pls_json_number(n),
        None => Err("No number found for port"),
    }
}
fn pls_json_number(number: &JsonValue) -> Result<u16, &'static str> {
    let num: f32 = match number {
        JsonValue::Number(n) => (*n).into(),
        _ => return Err("Invalid port range syntax, not a number"),
    };
    Ok(num as u16)
}
