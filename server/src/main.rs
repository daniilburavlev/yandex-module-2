extern crate core;

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

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
    let data = fs::read_to_string(&cli.tickers_path).expect("Failed to read 'tickers' file");
    let tickers = data.lines().map(|s| s.to_string()).collect();
    start(cli.tcp_port, cli.udp_port, tickers);
}

fn start(tcp_port: u16, udp_port: u16, tickers: Vec<String>) {
    let Ok(command_rx) = tcp::run(&format!("127.0.0.1:{}", tcp_port)) else {
        eprintln!("Unable to bind TCP socket");
        exit(-1);
    };
    let (stock_tx, stock_rx) = crossbeam::channel::unbounded();
    generator::run(tickers, stock_tx.clone());

    if let Err(e) = udp::run(udp_port, command_rx, stock_tx, stock_rx) {
        eprintln!("Error: {}", e);
        exit(-1);
    }
}
