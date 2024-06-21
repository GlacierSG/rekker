use std::io::{self, Read, Write, Result, Error};
use std::cmp::min;
use colored::*;
use crate::to_lit_colored;
use colored::Colorize;
use std::mem;
use chrono::prelude::*;

fn now() -> String {
    Utc::now().format("%H:%M:%S").to_string()
}

#[derive(Debug)]
pub struct WriteBuffer<T: Write> {
    stream: T,
    pub logging: bool,
}

macro_rules! impl_send_func {
    () => {
        pub fn write_all(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
            let msg = msg.as_ref();
            if self.logging && msg.len() != 0 {
                eprintln!("{} {} {}", now().red().bold(), "->".red().bold(), to_lit_colored(&msg, |x| x.normal(), |x| x.green()));
            }

            self.stream.write_all(&msg)
        }
        pub fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
            let msg = msg.as_ref();
            self.write_all(msg)
        }
    }
}

impl<T: Write> WriteBuffer<T> {
    pub fn from_stream(stream: T, logging: bool) -> Self {
        Self {
            stream: stream,
            logging: logging
        }
    }

    impl_send_func!();

}

#[derive(Debug)]
pub struct Buffer<T: Read + Write> {
    pub stream: T,
    buf: Vec<u8>,
    pub logging: bool,
}


impl<T: Read + Write> Buffer<T> {
    pub fn new(stream: T) -> Self {
        Buffer {
            stream: stream,
            buf: vec![],
            logging: false,
        }
    }

    fn read_to_buf(&mut self) -> io::Result<usize> {
        let mut buf = vec![0; 65535];
        let cap = self.stream.read(&mut buf)?;
        if cap == 0 {
             return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Pipe Broke",
                ));
        }

        if self.logging {
            eprintln!("{} {} {}", now().red().bold(), "<-".red().bold(), to_lit_colored(&buf[..cap], |x| x.normal(), |x| x.yellow()));
        }

        self.buf.extend(&buf[..cap]);
        Ok(self.buf.len())
    }
    fn read_all_to_buffer(&mut self) -> Result<usize> {
        let mut buffer = vec![];
        self.stream.read_to_end(&mut buffer)?;

        if self.logging && buffer.len() != 0 {
            eprintln!("{} {} {}", now().red().bold(), "<-".red().bold(), to_lit_colored(&buffer, |x| x.normal(), |x| x.yellow()));
        }

        self.buf.extend(buffer);
        Ok(self.buf.len())
    }
    impl_send_func!();

}

impl<R: Read + Write> Buffer<R> {
    fn drain_n(&mut self, size: usize) -> Vec<u8> {
        let out = self.buf[..size].to_vec();
        self.buf.drain(..size);
        return out;
    }

    pub fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        if self.buf.len() > 0 { 
            let m = min(self.buf.len(), size);
            return Ok(self.drain_n(m))
        }

        let m = min(self.read_to_buf()?, size);
        
        Ok(self.drain_n(m))
    }
    pub fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        if self.buf.len() >= size { 
            return Ok(self.drain_n(size))
        }
        while self.read_to_buf()? < size {}

        Ok(self.drain_n(size))
    }
    pub fn recvline(&mut self) -> Result<Vec<u8>> {
        if self.buf.len() > 0 { 
            for j in 0..self.buf.len() {
                if self.buf[j] == 10 {
                    return Ok(self.drain_n(j+1))
                }
            }
        }
        let mut i = 0;
        loop {
            let n = self.read_to_buf()?;
            for j in i..n {
                if self.buf[j] == 10 {
                    return Ok(self.drain_n(j+1))
                }
            }
            i = n;
        }

    }
    pub fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let suffix = suffix.as_ref();
        if suffix.len() == 0 {
            return Ok(vec![])
        }

        let mut i = 0;
        let mut n = self.buf.len();
        loop {
            for j in i..n {
                if self.buf[j] == suffix[suffix.len()-1] {
                    if suffix.len() <= self.buf.len() && j >= suffix.len()-1 && suffix == &self.buf[j+1-suffix.len()..j+1] {
                        return Ok(self.drain_n(j+1));
                    }
                }
            }
            i = n;
            n = self.read_to_buf()?;
        }
    }
    pub fn recvall(&mut self) -> Result<Vec<u8>> {
        self.read_all_to_buffer()?;

        Ok(mem::take(&mut self.buf))
    }

    pub fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let msg = msg.as_ref();
        let mut buffer = Vec::with_capacity(msg.len()+1);
        buffer.extend_from_slice(&msg);
        buffer.push(b'\n');
        self.send(buffer)?;
        Ok(())
    }
    pub fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let buf = self.recvuntil(suffix)?;
        self.sendline(msg)?;
        Ok(buf)
    }
}

