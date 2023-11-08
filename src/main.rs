// Jackson Coxson

use std::{net::Ipv4Addr, str::FromStr};

mod tcp;
mod udp;

static USAGE: &str = "swarm_proxy <target> udp [<udp_port|udp_range>] tcp [<tcp_port|tcp_range>]";

#[tokio::main]
async fn main() {
    env_logger::init();
    // Get the requested proxy settings from the args
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("{USAGE}");
        panic!("Not enough arguments!")
    }
    let mut udp_ports = vec![];
    let mut tcp_ports = vec![];

    let target = args[1].clone();
    let target = Ipv4Addr::from_str(&target).expect("Unable to parse target address as IPv4");

    enum Parse {
        Udp,
        Tcp,
        None,
    }
    let mut parse_mode = Parse::None;
    for arg in &args[2..] {
        match arg.as_str() {
            "udp" => {
                parse_mode = Parse::Udp;
            }
            "tcp" => {
                parse_mode = Parse::Tcp;
            }
            _ => {
                // Args can either be a single port or a range with :
                let port_range = arg.split(':').collect::<Vec<&str>>();
                if port_range.len() == 2 {
                    let start = port_range[0].parse::<u16>().unwrap();
                    let end = port_range[1].parse::<u16>().unwrap();
                    for i in start..=end {
                        match parse_mode {
                            Parse::Udp => udp_ports.push(i),
                            Parse::Tcp => tcp_ports.push(i),
                            Parse::None => {
                                println!("{USAGE}");
                                panic!("UDP or TCP wasn't selected")
                            }
                        }
                    }
                } else {
                    match parse_mode {
                        Parse::Udp => udp_ports.push(arg.parse::<u16>().unwrap()),
                        Parse::Tcp => tcp_ports.push(arg.parse::<u16>().unwrap()),
                        Parse::None => {
                            println!("{USAGE}");
                            panic!("UDP or TCP wasn't selected")
                        }
                    }
                }
            }
        }
    }

    if udp_ports.is_empty() && tcp_ports.is_empty() {
        println!("{USAGE}");
        panic!("No ports supplied");
    }

    let mut tasks = tokio::task::JoinSet::new();
    for udp_task in udp_ports {
        tasks.spawn(udp::open_udp(udp_task, target));
    }
    for tcp_task in tcp_ports {
        tasks.spawn(tcp::open_tcp(tcp_task, target));
    }
    println!("Starting server, all ports have been requested!");
    loop {
        let i = tasks.join_next().await;
        if i.is_none() {
            break;
        }
        println!("Task {:?} crashed!", i);
    }
}
