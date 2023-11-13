// Jackson Coxson

mod config;
mod tcp;
mod udp;

static USAGE: &str =
    "swarm_proxy <config.json> | <target> udp [<udp_port|udp_range>] tcp [<tcp_port|tcp_range>]";

#[tokio::main]
async fn main() {
    env_logger::init();
    // Generate the config
    let config =
        config::Configs::load().unwrap_or_else(|_| panic!("\nFailed to load config!\n{USAGE}\n"));

    let mut tasks = tokio::task::JoinSet::new();
    for (host, remote) in config.udp {
        tasks.spawn(udp::open_udp(host, remote));
    }
    for (host, remote) in config.tcp {
        tasks.spawn(tcp::open_tcp(host, remote));
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
