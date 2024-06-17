use std::net::{TcpListener};
use std::io::Result;
use super::tcp::Tcp;
use regex::Regex;

pub struct TcpListen {
    listener: TcpListener,
}

impl TcpListen {
    pub fn new(addr: &str) -> Result<Self> {
        let re = Regex::new(r"\s+").unwrap();
        let addr = re.replace_all(addr.trim(), ":");

        let listener = TcpListener::bind(addr.as_ref())?;
        Ok(TcpListen { listener: listener })
    }

    pub fn accept(&self) -> Result<(Tcp, String)> {
        let (stream, addr) = self.listener.accept()?;
        let mut stream = Tcp::from_stream(stream)?;
        let _ = stream.set_nagle(false);
        Ok((stream, addr.to_string()))
    }
}


