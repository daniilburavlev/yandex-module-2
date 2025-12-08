extern crate core;

use clap::Parser;
use log::info;
use std::path::PathBuf;
use std::process::exit;
use std::{fs, io};

mod generator;
mod tcp;
mod udp;

#[derive(Parser, Debug)]
#[clap(
    version,
    author,
    about,
    long_about = "CLI утилита для сравнения выписок из двух файлов"
)]
struct Cli {
    #[clap(long)]
    #[arg(default_value = "8080")]
    tcp_port: u16,
    #[clap(long)]
    #[arg(default_value = "7867")]
    udp_port: u16,
    #[clap(long)]
    #[arg(default_value = "resources/tickers.txt")]
    tickers_path: PathBuf,
}

fn main() {
    let cli = Cli::parse();
    env_logger::init();
    if let Err(e) = start(cli.tcp_port, cli.udp_port, cli.tickers_path) {
        eprintln!("{}", e);
        exit(-1);
    }
}

fn start(tcp_port: u16, udp_port: u16, tickers_path: PathBuf) -> io::Result<()> {
    let data = fs::read_to_string(tickers_path)?;
    let tickers = quotes::parse_tickers(&data);

    info!("Starting server on TCP:{}/UDP:{}", tcp_port, udp_port);
    let command_rx = tcp::run(&format!("127.0.0.1:{}", tcp_port))?;
    let (stock_tx, stock_rx) = crossbeam::channel::unbounded();
    generator::run(tickers, stock_tx.clone());

    udp::run(udp_port, command_rx, stock_tx, stock_rx)?;
    Ok(())
}
