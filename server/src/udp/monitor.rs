use crate::udp::client::ClientCommand;
use crossbeam::channel::Sender;
use log::error;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(5);

type KeepAliveHolder = Arc<Mutex<HashMap<SocketAddr, Instant>>>;

pub(crate) struct ClientsMonitor {
    socket: UdpSocket,
    clients: KeepAliveHolder,
    stop_rx: Sender<ClientCommand>,
}

impl ClientsMonitor {
    pub(crate) fn run(
        socket: UdpSocket,
        stop_tx: Sender<ClientCommand>,
    ) -> mpsc::Sender<SocketAddr> {
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        let clients_holder = Arc::new(Mutex::new(HashMap::new()));
        let mut monitoring = Self::new(socket, stop_tx.clone(), Arc::clone(&clients_holder));

        let check_holder = Arc::clone(&clients_holder);

        thread::spawn(move || {
            monitoring.start();
        });

        thread::spawn(move || {
            while let Ok(new_client_addr) = rx.recv() {
                let mut holder = clients_holder.lock().unwrap();
                holder.insert(new_client_addr, Instant::now());
            }
        });

        thread::spawn(move || {
            loop {
                thread::sleep(KEEPALIVE_INTERVAL);
                let mut holder = check_holder.lock().unwrap();
                let mut to_remove = Vec::new();
                for (k, v) in holder.iter() {
                    if *v + KEEPALIVE_INTERVAL < Instant::now() {
                        to_remove.push(*k);
                        if stop_tx.send(ClientCommand::Stop(*k)).is_err() {
                            break;
                        }
                    }
                }
                for k in to_remove {
                    holder.remove(&k);
                }
            }
        });
        tx
    }

    fn new(socket: UdpSocket, stop_rx: Sender<ClientCommand>, clients: KeepAliveHolder) -> Self {
        Self {
            socket,
            clients,
            stop_rx,
        }
    }

    fn start(&mut self) {
        loop {
            let mut buffer = [0u8; 6];
            match self.socket.recv_from(&mut buffer) {
                Ok((size, addr)) => {
                    if size != 4 || String::from_utf8_lossy(&buffer[..size]) != "PING" {
                        continue;
                    }
                    let Ok(mut clients) = self.clients.lock() else {
                        continue;
                    };
                    let stock = clients.entry(addr).or_insert_with(Instant::now);
                    if Instant::now() + KEEPALIVE_INTERVAL < *stock {
                        clients.remove(&addr);
                        if self.stop_rx.send(ClientCommand::Stop(addr)).is_err() {
                            error!("Channel closed");
                            break;
                        }
                    }
                    self.socket.send_to(b"PONG", addr).unwrap();
                }
                Err(e) => {
                    error!("Error receiving from socket: {}", e);
                }
            }
        }
    }
}
