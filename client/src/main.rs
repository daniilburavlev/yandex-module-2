mod client;
mod monitor;
mod server;

use crate::server::Server;
use clap::Parser;
use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc;
use std::{fs, io, thread};

#[derive(Parser)]
struct Cli {
    #[clap(long)]
    #[arg(default_value = "127.0.0.1:8080")]
    remote_addr: SocketAddr,
    #[clap(long)]
    #[arg(default_value = "127.0.0.1:9090")]
    local_addr: SocketAddr,
    #[clap(long)]
    #[arg(default_value = "resources/sub.txt")]
    tickers: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    env_logger::init();
    if let Err(e) = start(cli.remote_addr, cli.local_addr, cli.tickers) {
        eprintln!("{}", e);
        exit(-1);
    }
}

fn start(remote_addr: SocketAddr, local_addr: SocketAddr, tickers: PathBuf) -> io::Result<()> {
    let tickers = load_tickers(tickers)?;

    let socket = UdpSocket::bind(local_addr)?;

    let (stock_tx, stock_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    let (addr_tx, pong_tx) = monitor::run(socket.try_clone()?, stop_tx.clone());
    Server::run(socket.try_clone()?, addr_tx, stock_tx, pong_tx, stop_tx)?;

    client::sub(local_addr, remote_addr, tickers)?;

    thread::spawn(move || {
        while let Ok(stock) = stock_rx.recv() {
            println!(
                "[{}] price: {} volume: {} timestamp: {}",
                stock.ticker, stock.price, stock.volume, stock.timestamp
            );
        }
    });
    if let Ok(error_msg) = stop_rx.recv() {
        return Err(io::Error::new(io::ErrorKind::Interrupted, error_msg));
    }
    Ok(())
}

fn load_tickers(path: PathBuf) -> io::Result<Vec<String>> {
    let data = fs::read_to_string(path)?;
    Ok(quotes::parse_tickers(&data))
}
