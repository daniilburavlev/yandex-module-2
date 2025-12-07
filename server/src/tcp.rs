use core::fmt;
use std::fmt::Formatter;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{io, thread};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Sub {
        address: SocketAddr,
        tickers: Vec<String>,
    },
}

impl FromStr for Command {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split_whitespace();
        let Some(command) = parts.next() else {
            return Err(bad_request(s));
        };
        match command {
            "SUB" => {
                let address = parts.next().ok_or(bad_request(s))?.to_string();
                let address = SocketAddr::from_str(address.as_str()).map_err(|_| bad_request(s))?;
                let tickers = parts
                    .next()
                    .ok_or(bad_request(s))?
                    .split(",")
                    .map(ToString::to_string)
                    .collect();
                Ok(Command::Sub { address, tickers })
            }
            _ => Err(bad_request(s)),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Command::Sub { address, tickers } => {
                f.write_fmt(format_args!("SUB {} {}", address, tickers.join(",")))?;
            }
        }
        Ok(())
    }
}

fn bad_request(s: &str) -> io::Error {
    io::Error::new(
        ErrorKind::InvalidInput,
        format!(
            "Bad request: {}, (example 'SUB udp://127.0.0.1:8080 TIC,TIC,TIC')",
            s
        ),
    )
}

pub(crate) enum Response {
    Ok,
    Err(String),
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => write!(f, "OK\r\n")?,
            Response::Err(e) => write!(f, "ERR {}\r\n", e)?,
        }
        Ok(())
    }
}

pub(crate) fn run(address: &str) -> io::Result<Receiver<Command>> {
    let listener = TcpListener::bind(address)?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let tx = tx.clone();
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        handle_stream(tx, stream);
                    });
                }
                Err(_) => {
                    eprintln!("Unable to accept TCP connection");
                    break;
                },
            }
        }
    });
    Ok(rx)
}

fn handle_stream(tx: Sender<Command>, stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    let Ok(_) = reader.read_line(&mut line) else {
        return;
    };
    let Ok(mut writer) = stream.try_clone() else {
        return;
    };
    match line.parse::<Command>() {
        Ok(command) => {
            let _ = tx.send(command);
            let _ = writer.write_all(Response::Ok.to_string().as_bytes());
            let _ = writer.flush();
        }
        Err(e) => {
            let _ = writer.write_all(Response::Err(e.to_string()).to_string().as_bytes());
            let _ = writer.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_serialization_deserialization() {
        let command = Command::Sub {
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            tickers: vec!["AAPL".to_string()],
        };
        let value = command.to_string();
        let result = value.parse::<Command>().unwrap();
        assert_eq!(result, command);
    }

    #[test]
    fn test_run_receive_command() {
        let port: i32 = rand::rng().random_range(8000..9000);
        let address = format!("127.0.0.1:{}", port);

        let rx = run(&address).unwrap();

        let tickers = vec!["AAPL".to_string()];
        let command = Command::Sub {
            address: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            tickers,
        };

        let mut stream = TcpStream::connect(&address).unwrap();
        let _ = stream.write_all(command.to_string().as_bytes());
        let _ = stream.write_all(b"\r\n");
        let _ = stream.flush();

        let result = rx.recv().unwrap();

        assert_eq!(command, result);
    }
}
