// Jackson Coxson

use std::net::Ipv4Addr;

use tokio::net::{TcpListener, TcpStream};

pub async fn open_tcp(port: u16, target: Ipv4Addr) {
    let socket = TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    loop {
        if let Ok((mut stream, src)) = socket.accept().await {
            println!("Accepted TCP connection from {:?}", src);
            tokio::spawn(async move {
                let mut remote = TcpStream::connect((target, port)).await.unwrap();
                if (tokio::io::copy_bidirectional(&mut stream, &mut remote).await).is_err() {
                    println!("Bidirectional TCP transfer closed");
                }
            });
        }
    }
}
