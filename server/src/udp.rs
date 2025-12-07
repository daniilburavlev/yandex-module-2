use crossbeam::channel::Receiver;
use quotes::StockQuote;
use std::collections::HashSet;
use std::net::UdpSocket;
use std::{io, thread};

pub(crate) enum ClientCommand {
    Send(StockQuote),
    Stop(String),
}

pub(crate) struct Client {
    socket: UdpSocket,
    address: String,
    tickers: HashSet<String>,
    stock_rx: Receiver<ClientCommand>,
}

impl Client {
    pub(crate) fn run(
        socket: UdpSocket,
        address: String,
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
        address: String,
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
                        let stock: Vec<u8> = stock.try_into()?;
                        self.socket.send_to(&stock, &self.address)?;
                    }
                }
                ClientCommand::Stop(address) => {
                    if address == self.address {
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
    use std::time::Duration;

    #[test]
    fn test_send_command() {
        let server = format!("127.0.0.1:{}", random_range::<i32, _>(8000..9000));

        let udp = UdpSocket::bind(&server).unwrap();
        let (tx, rx) = crossbeam::channel::unbounded();

        let client = format!("127.0.0.1:{}", random_range::<i32, _>(8000..9000));
        let mut tickers = HashSet::new();
        tickers.insert(String::from("AAPL"));
        Client::run(udp, client.clone(), tickers, rx).unwrap();
        let client = UdpSocket::bind(client).unwrap();

        thread::sleep(Duration::from_millis(100));

        let stock = StockQuote::new("AAPL", 100, 100);
        tx.send(ClientCommand::Send(stock.clone())).unwrap();

        let mut buffer = [0u8; 1024];
        client.recv(&mut buffer).unwrap();

        let buffer = buffer.to_vec();
        let result: StockQuote = buffer.try_into().unwrap();

        assert_eq!(stock, result);
    }
}
