use log::{error, info};
use quotes::StockQuote;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::Sender;
use std::{io, thread};

const BUFFER_SIZE: usize = 1024;
const PONG_SIZE: usize = 4;

pub(crate) struct Server {
    addr_tx: Sender<SocketAddr>,
    stock_tx: Sender<StockQuote>,
    pong_tx: Sender<()>,
    socket: UdpSocket,
    stop_tx: Sender<String>,
}

impl Server {
    pub(crate) fn run(
        socket: UdpSocket,
        addr_tx: Sender<SocketAddr>,
        stock_tx: Sender<StockQuote>,
        pong_tx: Sender<()>,
        stop_tx: Sender<String>,
    ) -> io::Result<()> {
        let server = Self::new(socket, addr_tx, stock_tx, pong_tx, stop_tx)?;

        thread::spawn(move || {
            server.start();
        });
        Ok(())
    }

    fn new(
        socket: UdpSocket,
        addr_tx: Sender<SocketAddr>,
        stock_tx: Sender<StockQuote>,
        pong_tx: Sender<()>,
        stop_tx: Sender<String>,
    ) -> io::Result<Self> {
        Ok(Self {
            socket,
            addr_tx,
            stock_tx,
            pong_tx,
            stop_tx,
        })
    }

    fn start(&self) {
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut address_sent = false;
        while let Ok((size, addr)) = self.socket.recv_from(&mut buffer) {
            if !address_sent {
                if self.addr_tx.send(addr).is_err() {
                    let _ = self
                        .stop_tx
                        .send("UDP socket address channel is closed".to_string());
                }
                address_sent = true;
            }
            if size == PONG_SIZE && String::from_utf8_lossy(&buffer[..PONG_SIZE]).eq("PONG") {
                info!("Received PONG from {}", addr);
                let _ = self.pong_tx.send(());
            } else {
                let Ok(stock) = serde_json::from_slice(&buffer[..size]) else {
                    error!("Invalid UDP packet!");
                    continue;
                };
                if self.stock_tx.send(stock).is_err() {
                    error!("Stock channel is closed!");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_server_run() {
        let address = SocketAddr::from(([127, 0, 0, 1], 9458));
        let socket = UdpSocket::bind(address).unwrap();
        let (addr_tx, addr_rx) = mpsc::channel();
        let (stock_tx, stock_rx) = mpsc::channel();
        let (pong_tx, pong_rx) = mpsc::channel();
        let (stop_tx, _) = mpsc::channel();

        Server::run(socket, addr_tx, stock_tx, pong_tx, stop_tx).unwrap();

        let sender_addr = SocketAddr::from(([127, 0, 0, 1], 9459));
        let socket = UdpSocket::bind(sender_addr).unwrap();
        socket.send_to(b"PONG", address).unwrap();

        let received_addr = addr_rx.recv().unwrap();
        assert_eq!(sender_addr, received_addr);

        let _ = pong_rx.recv().unwrap();

        let stock = StockQuote::new("AAPL", 200, 3000000);
        let stock_json = serde_json::to_vec(&stock).unwrap();
        socket.send_to(&stock_json, address).unwrap();
        let received_stock = stock_rx.recv().unwrap();
        assert_eq!(received_stock, stock);
    }
}
