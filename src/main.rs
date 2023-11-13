// Jackson Coxson

mod config;
mod tcp;
mod udp;

static USAGE: &str = "swarm_proxy <config.json> | <target> [<mode> [<ports>]]";

#[tokio::main]
async fn main() {
    env_logger::init();
    // Generate the config
    let config = match config::Configs::load() {
        Ok(c) => c,
        Err(e) => {
            println!("Error loading config: {}", e);
            println!("USAGE: {USAGE}");
            return;
        }
    };

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
