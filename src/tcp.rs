// Jackson Coxson

use std::net::SocketAddrV4;

use log::info;
use tokio::net::{TcpListener, TcpStream};

pub async fn open_tcp(host: SocketAddrV4, remote: SocketAddrV4) {
    let socket = TcpListener::bind(host).await.unwrap();
    info!("Listening TCP on {:?}", host);
    loop {
        if let Ok((mut stream, src)) = socket.accept().await {
            println!("Accepted TCP connection from {:?}", src);
            tokio::spawn(async move {
                let mut remote = TcpStream::connect(remote).await.unwrap();
                if (tokio::io::copy_bidirectional(&mut stream, &mut remote).await).is_err() {
                    println!("Bidirectional TCP transfer closed");
                }
            });
        }
    }
}
