use crossbeam::channel::Receiver;
use log::{error, info};
use quotes::StockQuote;
use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::{io, thread};

#[derive(Debug)]
pub enum ClientCommand {
    Send(StockQuote),
    Stop(SocketAddr),
}

pub(crate) struct Client {
    socket: UdpSocket,
    address: SocketAddr,
    tickers: HashSet<String>,
    stock_rx: Receiver<ClientCommand>,
}

impl Client {
    pub(crate) fn run(
        socket: UdpSocket,
        address: SocketAddr,
        tickers: HashSet<String>,
        stock_rx: Receiver<ClientCommand>,
    ) -> io::Result<()> {
        let client = Client::new(socket, address, tickers, stock_rx)?;
        thread::spawn(move || {
            client.start().expect("Client error");
        });
        Ok(())
    }

    fn new(
        socket: UdpSocket,
        address: SocketAddr,
        tickers: HashSet<String>,
        stock_rx: Receiver<ClientCommand>,
    ) -> io::Result<Self> {
        Ok(Self {
            socket,
            address,
            tickers,
            stock_rx,
        })
    }

    fn start(&self) -> io::Result<()> {
        while let Ok(command) = self.stock_rx.recv() {
            match command {
                ClientCommand::Send(stock) => {
                    if self.tickers.contains(&stock.ticker) {
                        match serde_json::to_vec(&stock) {
                            Ok(stock) => {
                                if let Err(e) = self.socket.send_to(&stock, self.address) {
                                    error!("Failed to send stock: {}", e);
                                    break;
                                }
                            }
                            Err(e) => error!("JSON error: {}", e),
                        }
                    }
                }
                ClientCommand::Stop(address) => {
                    if address == self.address {
                        info!("subscriber {} stopped", self.address);
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::random_range;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    #[test]
    fn test_send_command() {
        let server = format!("127.0.0.1:{}", random_range::<i32, _>(8000..9000));

        let udp = UdpSocket::bind(&server).unwrap();
        let (tx, rx) = crossbeam::channel::unbounded();

        let client = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            random_range(8000..9000),
        );
        let mut tickers = HashSet::new();
        tickers.insert(String::from("AAPL"));
        Client::run(udp, client.clone(), tickers, rx).unwrap();
        let client = UdpSocket::bind(client).unwrap();

        thread::sleep(Duration::from_millis(100));

        let stock = StockQuote::new("AAPL", 100, 100);
        tx.send(ClientCommand::Send(stock.clone())).unwrap();

        let mut buffer = [0u8; 2048];
        let len = client.recv(&mut buffer).unwrap();

        let result: StockQuote = serde_json::from_slice(&buffer[..len]).unwrap();

        assert_eq!(stock, result);
    }
}
