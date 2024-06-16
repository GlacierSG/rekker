use std::net::{TcpListener, TcpStream};
use std::io::{Result, Error};
use super::tcp::Tcp;
use std::io::{Read, Write};

pub struct TcpListen {
    listener: TcpListener,
}

impl TcpListen {
    pub fn new(address: &str) -> Result<Self> {
        let listener = TcpListener::bind(address)?;
        Ok(TcpListen { listener: listener })
    }

    pub fn accept(&self) -> Result<(Tcp, String)> {
        let (stream, addr) = self.listener
            .accept()?;
        Ok((Tcp::from_stream(stream)?, addr.to_string()))
    }
}


