// Jackson Coxson

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use log::{info, warn};
use tokio::net::UdpSocket;

use kanal::{unbounded_async, AsyncSender};

const BUFFER_SIZE: usize = 2048;

struct SocketMap {
    internal: HashMap<SocketAddrV4, AsyncSender<Vec<u8>>>,
    target: SocketAddrV4,
    master_tx: AsyncSender<(Vec<u8>, SocketAddrV4)>,
}

impl SocketMap {
    fn new(target: SocketAddrV4, master_tx: AsyncSender<(Vec<u8>, SocketAddrV4)>) -> SocketMap {
        SocketMap {
            internal: HashMap::new(),
            target,
            master_tx,
        }
    }
    async fn get(&mut self, socket: &SocketAddrV4) -> AsyncSender<Vec<u8>> {
        if let Some(slave) = self.internal.get_mut(socket) {
            if slave.is_disconnected() {
                let tx = udp_slave(self.target, *socket, self.master_tx.clone()).await;
                self.internal.insert(*socket, tx.clone());
                tx
            } else {
                slave.clone()
            }
        } else {
            let tx = udp_slave(self.target, *socket, self.master_tx.clone()).await;
            self.internal.insert(*socket, tx.clone());
            tx
        }
    }
    fn clear(&mut self) {
        self.internal.clear();
    }
}

pub async fn open_udp(port: u16, target: Ipv4Addr) {
    let socket = UdpSocket::bind(("0.0.0.0", port)).await.unwrap();
    info!("Listening on UDP port {port}");

    let (master_tx, master_rx) = unbounded_async();

    let mut slaves = SocketMap::new(SocketAddrV4::new(target, port), master_tx);
    loop {
        let mut buf = [0u8; BUFFER_SIZE];
        tokio::select! {
            res = socket.recv_from(&mut buf) => {
                if let Ok((size, src)) = res {
                    info!("Received {size} bytes from {:?}", src);
                    match src {
                        SocketAddr::V4(src) => {
                            let slave = slaves.get(&src).await;
                            slave.send(buf[..size].to_vec()).await.unwrap();
                        }
                        SocketAddr::V6(_) => {
                            println!("IPv6 is unimplemented smh nobody asked nobody cares");
                        }
                    }
                }
            }
            msg = master_rx.recv() => {
                if let Ok((msg, remote)) = msg {
                    info!("Sending {:?} bytes to {:?}", msg.len(), remote);
                    socket.send_to(&msg, remote).await.unwrap();
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(310)) => {
                slaves.clear();
            }
        }
    }
}

async fn udp_slave(
    target: SocketAddrV4,
    remote: SocketAddrV4,
    sender: AsyncSender<(Vec<u8>, SocketAddrV4)>,
) -> AsyncSender<Vec<u8>> {
    let (tx, rx) = unbounded_async::<Vec<u8>>();

    let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
    info!("UDP slave bound to {:?}", socket.local_addr());

    tokio::spawn(async move {
        let mut buf = [0u8; BUFFER_SIZE];
        loop {
            tokio::select! {
                buf = rx.recv() => {
                    if let Ok(buf) = buf {
                        info!("Sending {:?} bytes to {:?}", buf.len(), target);
                        socket.send_to(&buf, &target).await.unwrap();
                    } else {
                        break;
                    }
                }
                res = socket.recv_from(&mut buf) => {
                    if let Ok((size, sauce)) = res {
                        info!("Received {size} bytes from {:?}", sauce);
                        sender.send((buf[..size].to_vec(), remote)).await.unwrap();
                    }

                }
                _ = tokio::time::sleep(Duration::from_secs(300)) => {
                    warn!("UDP slave for {:?} has been reaped", remote);
                    break;
                }
            }
        }
    });

    tx
}
