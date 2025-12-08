use log::info;
use std::io;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

const BUFFER_SIZE: usize = 1024;

pub(crate) fn sub(addr: SocketAddr, remote: SocketAddr, tickers: Vec<String>) -> io::Result<()> {
    let mut stream = TcpStream::connect(remote)?;
    let request = format!("SUB {} {}\r\n", addr, tickers.join(","));
    info!("Sending request to {}: {}", remote, request);
    let _ = stream.write(request.as_bytes())?;
    stream.flush()?;
    let mut buffer = [0u8; BUFFER_SIZE];
    let size = stream.read(&mut buffer)?;
    let buffer = &buffer[..size];
    if size == 2 && buffer == b"OK" {
        info!("Subscribed to: {}", tickers.join(","));
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            String::from_utf8_lossy(buffer),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::str::FromStr;
    use std::thread;

    #[test]
    fn test_sub() {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut buffer = [0u8; 1024];
                let size = stream.read(&mut buffer).unwrap();
                assert_eq!(&buffer[..size], b"SUB 127.0.0.1:9090 AAPL\r\n");
                stream.write_all(b"OK\r\n").unwrap();
            }
        });
        sub(
            SocketAddr::from_str("127.0.0.1:9090").unwrap(),
            SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            vec!["AAPL".to_string()],
        )
        .unwrap();
    }
}
