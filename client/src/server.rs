use log::error;
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
                    self.stop_tx
                        .send("UDP socket address channel is closed".to_string())
                        .unwrap();
                }
                address_sent = true;
            }
            if size == PONG_SIZE && String::from_utf8_lossy(&buffer[..PONG_SIZE]).eq("PONG") {
                let _ = self.pong_tx.send(());
            } else {
                let Ok(stock) = buffer.to_vec().try_into() else {
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
