use crossbeam::channel::{Receiver, Sender};
use quotes::StockQuote;
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
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

type ClientsHolder = Arc<Mutex<HashMap<SocketAddr, Instant>>>;

struct ClientsMonitoring {
    socket: UdpSocket,
    clients: ClientsHolder,
    stop_rx: Sender<ClientCommand>,
}

impl ClientsMonitoring {
    pub(crate) fn run(
        socket: UdpSocket,
        stop_rx: Sender<ClientCommand>,
    ) -> mpsc::Sender<SocketAddr> {
        let (tx, rx) = mpsc::channel::<SocketAddr>();
        let clients_holder = Arc::new(Mutex::new(HashMap::new()));
        let mut monitoring = Self::new(socket, stop_rx, Arc::clone(&clients_holder));
        thread::spawn(move || {
            monitoring.start();
        });
        thread::spawn(move || {
            while let Ok(new_client_addr) = rx.recv() {
                let mut holder = clients_holder.lock().unwrap();
                holder.insert(new_client_addr, Instant::now());
            }
        });
        tx
    }

    fn new(socket: UdpSocket, stop_rx: Sender<ClientCommand>, clients: ClientsHolder) -> Self {
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
                Ok((_, addr)) => {
                    let Ok(mut clients) = self.clients.lock() else {
                        continue;
                    };
                    let stock = clients.entry(addr).or_insert_with(Instant::now);
                    if Instant::now() + Duration::from_secs(5) < *stock {
                        clients.remove(&addr);
                        if self
                            .stop_rx
                            .send(ClientCommand::Stop(addr.to_string()))
                            .is_err()
                        {
                            eprintln!("Channel closed");
                            break;
                        }
                    }
                    self.socket.send_to(b"PONG", &addr).unwrap();
                }
                Err(e) => {
                    eprintln!("Error receiving from socket: {}", e);
                }
            }
        }
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
