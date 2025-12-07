use crate::tcp::Command;
use crate::udp::client::Client;
use crossbeam::channel::{Receiver, Sender};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;

mod client;
mod monitor;

pub use crate::udp::client::ClientCommand;
use crate::udp::monitor::ClientsMonitor;

pub(crate) fn run(
    port: u16,
    command_rx: mpsc::Receiver<Command>,
    tx: Sender<ClientCommand>,
    rx: Receiver<ClientCommand>,
) -> io::Result<()> {
    let address = format!("127.0.0.1:{}", port);
    let socket = UdpSocket::bind(address)?;

    let new_client_tx = ClientsMonitor::run(socket.try_clone()?, tx);

    while let Ok(command) = command_rx.recv() {
        let rx = rx.clone();
        handle_command(socket.try_clone()?, command, rx, new_client_tx.clone());
    }

    Ok(())
}

fn handle_command(
    socket: UdpSocket,
    command: Command,
    rx: Receiver<ClientCommand>,
    new_client_tx: mpsc::Sender<SocketAddr>,
) {
    match command {
        Command::Sub { address, tickers } => {
            new_client_tx.send(address).unwrap();
            if let Err(e) = Client::run(socket, address, tickers.into_iter().collect(), rx) {
                eprintln!("{}", e);
            }
        }
    }
}
