use log::info;
use std::io;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};

pub(crate) fn sub(addr: SocketAddr, remote: SocketAddr, tickers: Vec<String>) -> io::Result<()> {
    let mut stream = TcpStream::connect(remote)?;
    let request = format!("SUB {} {}\r\n", addr, tickers.join(","));
    info!("Sending request to {}: {}", remote, request);
    stream.write(request.as_bytes())?;
    stream.flush()?;
    Ok(())
}
