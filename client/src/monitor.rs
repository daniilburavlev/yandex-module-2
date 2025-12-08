use log::{error, info};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

const SLEEP_DURATION: Duration = Duration::from_secs(2);
const TTL: Duration = Duration::from_secs(5);

pub(crate) fn run(socket: UdpSocket, stop_tx: Sender<String>) -> (Sender<SocketAddr>, Sender<()>) {
    let (addr_tx, addr_rx) = mpsc::channel();
    let (pong_tx, pong_rx) = mpsc::channel();

    let monitor = PingPongMonitor::new();
    monitor.run_ping(socket, addr_rx, stop_tx.clone());
    monitor.run_pong_handler(pong_rx, stop_tx);

    (addr_tx, pong_tx)
}

struct PingPongMonitor {}

impl PingPongMonitor {
    fn new() -> Self {
        Self {}
    }

    fn run_ping(&self, socket: UdpSocket, addr_rx: Receiver<SocketAddr>, stop_tx: Sender<String>) {
        thread::spawn(move || {
            let Ok(addr) = addr_rx.recv() else {
                if stop_tx
                    .send("Failed to get UDP address, channel is closed!".to_string())
                    .is_err()
                {
                    error!("Failed to send stop cmd, channel is closed!");
                }
                return;
            };
            loop {
                if socket.send_to(b"PING", addr).is_err() {
                    if stop_tx
                        .send("Failed to send 'PING' UDP socket is closed!".to_string())
                        .is_err()
                    {
                        error!("Failed to send stop cmd, channel is closed!");
                    }
                    break;
                }
                info!("Sending PING to: {}", addr);
                thread::sleep(SLEEP_DURATION);
            }
        });
    }

    fn run_pong_handler(&self, pong_rx: Receiver<()>, stop_tx: Sender<String>) {
        thread::spawn(move || {
            loop {
                match pong_rx.recv_timeout(TTL) {
                    Ok(()) => {}
                    Err(_) => {
                        stop_tx
                            .send(format!("Server not respond in {} seconds", TTL.as_secs()))
                            .expect("Failed to send to channel, closed");
                        break;
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{SocketAddr, UdpSocket};
    use std::str::FromStr;
    use std::sync::mpsc;

    #[test]
    fn test_run_ping() {
        let server = UdpSocket::bind("127.0.0.1:9191").expect("Failed to bind client socket");
        let client = UdpSocket::bind("127.0.0.1:9292").expect("Failed to bind client socket");
        let monitor = PingPongMonitor::new();
        let (addr_tx, addr_rx) = mpsc::channel();
        let (stop_tx, _) = mpsc::channel();
        monitor.run_ping(client, addr_rx, stop_tx);
        addr_tx
            .send(SocketAddr::from_str("127.0.0.1:9191").unwrap())
            .unwrap();
        let mut buffer = [0u8; 4];
        server.recv(&mut buffer).unwrap();
        assert_eq!("PING", std::str::from_utf8(&buffer).unwrap());
    }

    #[test]
    fn test_run_pong() {
        let monitor = PingPongMonitor::new();
        let (_, pong_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        monitor.run_pong_handler(pong_rx, stop_tx.clone());
        let error_msg = stop_rx.recv().unwrap();
        assert_eq!(
            format!("Server not respond in {} seconds", TTL.as_secs()),
            error_msg
        );
    }
}
