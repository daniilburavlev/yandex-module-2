use crate::udp::client::ClientCommand;
use crossbeam::channel::Sender;
use log::{debug, error};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::process::exit;
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(5);

type KeepAliveHolder = Arc<Mutex<HashMap<SocketAddr, Instant>>>;

pub(crate) struct ClientsMonitor {
    socket: UdpSocket,
    clients: KeepAliveHolder,
}

impl ClientsMonitor {
    pub(crate) fn run(
        socket: UdpSocket,
        stop_tx: Sender<ClientCommand>,
    ) -> mpsc::Sender<SocketAddr> {
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        let clients_holder = Arc::new(Mutex::new(HashMap::new()));
        let mut monitoring = Self::new(socket, Arc::clone(&clients_holder));

        let check_holder = Arc::clone(&clients_holder);

        thread::spawn(move || {
            monitoring.start();
        });

        thread::spawn(move || {
            while let Ok(new_client_addr) = rx.recv() {
                let mut holder = clients_holder.lock();
                holder.insert(new_client_addr, Instant::now());
            }
        });

        thread::spawn(move || {
            loop {
                thread::sleep(KEEPALIVE_INTERVAL);
                let mut holder = check_holder.lock();
                let mut to_remove = Vec::new();
                for (k, v) in holder.iter() {
                    if *v + KEEPALIVE_INTERVAL < Instant::now() {
                        debug!(
                            "{} not responding for: {}s",
                            k,
                            KEEPALIVE_INTERVAL.as_secs()
                        );
                        to_remove.push(*k);
                        if stop_tx.send(ClientCommand::Stop(*k)).is_err() {
                            error!("Stop channel is closed!");
                            exit(-1);
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

    fn new(socket: UdpSocket, clients: KeepAliveHolder) -> Self {
        Self { socket, clients }
    }

    fn start(&mut self) {
        loop {
            let mut buffer = [0u8; 6];
            match self.socket.recv_from(&mut buffer) {
                Ok((size, addr)) => {
                    if size != 4 || String::from_utf8_lossy(&buffer[..size]) != "PING" {
                        continue;
                    }
                    let mut clients = self.clients.lock();
                    clients.insert(addr, Instant::now());
                    if let Err(e) = self.socket.send_to(b"PONG", addr) {
                        error!("Server disconnected: {}", e);
                        exit(-1);
                    }
                }
                Err(e) => {
                    error!("Error receiving from socket: {}", e);
                }
            }
        }
    }
}
