use crate::tcp::Command;
use crate::udp::client::Client;
use crossbeam::channel::{Receiver, Sender};
use std::io;
use std::net::UdpSocket;
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
    
    ClientsMonitor::run(socket.try_clone()?, tx);

    while let Ok(command) = command_rx.recv() {
        let rx = rx.clone();
        handle_command(socket.try_clone()?, command, rx);
    }

    Ok(())
}

fn handle_command(socket: UdpSocket, command: Command, rx: Receiver<ClientCommand>) {
    match command {
        Command::Sub { address, tickers } => {
            if let Err(e) = Client::new(socket, address, tickers.into_iter().collect(), rx) {
                eprintln!("{}", e);
            }
        }
    }
}
