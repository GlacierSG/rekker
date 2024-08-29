use crate::{Result, Error};
use std::time::Duration;
use std::net::{TcpStream, TcpListener, UdpSocket};
use std::cmp::min;
use std::io::{self, Read, Write};
use chrono::prelude::*;
use crate::{to_lit_colored, from_lit};
use colored::Colorize;
use regex::Regex;
use rustls::{RootCertStore, ClientConnection, StreamOwned};
use rustls::pki_types::ServerName;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;

fn trim_addr(addr: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    let addr = re.replace_all(addr.trim(), ":");
    addr.to_string()
}

fn now() -> String {
    Utc::now().format("%H:%M:%S").to_string()
}

pub enum Listener {
    Tcp { listener: TcpListener, log: bool },
}

impl Listener {
    pub fn tcp(addr: &str) -> Result<Self> {
        let re = Regex::new(r"\s+").unwrap();
        let addr = re.replace_all(addr.trim(), ":");

        let listener = TcpListener::bind(addr.as_ref())?;
        Ok(Listener::Tcp { listener, log: false })
    }
    
    pub fn accept(&self) -> Result<(Pipe, String)> {
        match self {
            Listener::Tcp { listener, .. } => {
                let (stream, addr) = listener.accept()?;
                let stream = Pipe::from(stream);
                Ok((stream, addr.to_string()))
            },
        }
    }
}

pub enum Pipe {
    Tcp { stream: TcpStream, buffer: Vec<u8>, log: bool },
    Udp { stream: UdpSocket, buffer: Vec<u8>, log: bool },
    Tls { stream: StreamOwned<ClientConnection, TcpStream>, buffer: Vec<u8>, log: bool },
}

impl From<TcpStream> for Pipe {
    fn from(stream: TcpStream) -> Pipe {
        Pipe::Tcp { stream, buffer: vec![], log: false }
    }
}
impl From<UdpSocket> for Pipe {
    fn from(stream: UdpSocket) -> Pipe {
        Pipe::Udp { stream, buffer: vec![], log: false }
    }
}
impl From<StreamOwned<ClientConnection, TcpStream>> for Pipe {
    fn from(stream: StreamOwned<ClientConnection, TcpStream>) -> Pipe {
        Pipe::Tls { stream, buffer: vec![], log: false }
    }
}

impl Pipe {
    pub fn tcp(addr: &str) -> Result<Pipe> {
        let addr = trim_addr(addr);

        let stream = TcpStream::connect(addr)?;
    
        Ok(Self::Tcp{ stream, buffer: vec![], log: false })
    }

    pub fn udp(addr: &str) -> Result<Pipe> {
        let addr = trim_addr(addr);

        let stream = UdpSocket::bind("0.0.0.0:0")?; 
        stream.connect(addr)?;

        Ok(Pipe::Udp {
            stream: stream,
            buffer: vec![0; 0],
            log: false,
        })
    }
    pub fn tls(addr: &str) -> Result<Pipe> {
        let addr = trim_addr(addr);

        let domain: ServerName<'_>;

        let t1: Vec<&str> = addr.split(|b| b as u32 == 58).collect(); // split on ':'
        if let Some(t1) = t1.get(0) {
            if let Ok(t2) = (*t1).to_string().try_into() {
                domain = t2;
            }
            else {
                return Err(Error::ParsingError("Could not parse domain".to_string()));
            }
        }
        else {
            return Err(Error::ParsingError("Could not parse domain".to_string()));
        }

        let stream = TcpStream::connect(&addr)?;

        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Allow SSLKEYLOGFILE
        config.key_log = Arc::new(rustls::KeyLogFile::new());

        let client = ClientConnection::new(Arc::new(config), domain).unwrap();
        
        let tls_stream = StreamOwned::new(client, stream);

        Ok(Self::Tls { stream: tls_stream, buffer: vec![], log: false })
    }

    pub fn set_nagle(&mut self, value: bool) -> Result<()> {
        match self {
            Self::Tcp { stream, .. } => stream.set_nodelay(!value)?,
            Self::Udp { .. } => return Err(Error::Invalid("cannot set nagle for UDP".to_string())),
            Self::Tls { stream, .. } => stream.sock.set_nodelay(!value)?,
        };
        Ok(())
    }
    pub fn nagle(&mut self) -> Result<bool> {
        match self {
            Self::Tcp { stream, .. } => Ok(!stream.nodelay()?),
            Self::Udp { .. } => Err(Error::Invalid("cannot set nagle for UDP".to_string())),
            Self::Tls { stream, .. } => Ok(!stream.sock.nodelay()?),
        }
    }
    pub fn is_logging(&self) -> bool {
        match self {
            Self::Tcp { log, .. } => *log,
            Self::Udp { log, .. } => *log,
            Self::Tls { log, .. } => *log,
        }
    }
    pub fn log(&mut self, value: bool) {
        match self {
            Self::Tcp { log, .. } => *log = value,
            Self::Udp { log, .. } => *log = value,
            Self::Tls { log, .. } => *log = value,
        }
    }

    pub fn buffer_len(&self) -> usize {
        match self {
            Self::Tcp { buffer, .. } => buffer.len(),
            Self::Udp { buffer, .. } => buffer.len(),
            Self::Tls { buffer, .. } => buffer.len(),
        }
    }
    pub fn read_to_buffer(&mut self) -> Result<usize> {
        let mut buf = [0; 1024];
        let cap = match self {
            Self::Tcp { stream, .. } => stream.read(&mut buf)?,
            Self::Udp { stream, .. } => stream.recv(&mut buf)?,
            Self::Tls { stream, .. } => stream.read(&mut buf)?,
        };
        if cap == 0 {
             return Ok(0);
        }

        if self.is_logging() {
            eprintln!("{} {} {} ", now().red().bold(), "<-".red().bold(), to_lit_colored(&buf[..cap], |x| x.normal(), |x| x.yellow()));
        }

        let buffer = match self {
            Self::Tcp { buffer, .. } => buffer,
            Self::Udp { buffer, .. } => buffer,
            Self::Tls { buffer, .. } => buffer,
        };
        buffer.extend(&buf[..cap]);
        Ok(cap)
    }
    pub fn drain_n(&mut self, n: usize) -> Vec<u8> {
        if n == 0 { return vec![]; }

        let buffer = match self {
            Self::Tcp { buffer, .. } => buffer,
            Self::Udp { buffer, .. } => buffer,
            Self::Tls { buffer, .. } => buffer,
        };

        let mut new_buf = buffer.split_off(n);
        std::mem::swap(&mut new_buf, buffer);
        new_buf
    }

    pub fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        if self.buffer_len() > 0 { 
            let n = min(self.buffer_len(), size);
            return Ok(self.drain_n(n))
        }
        self.read_to_buffer()?;
        let n = min(self.buffer_len(), size);
        
        Ok(self.drain_n(n))
    }

    pub fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        if self.buffer_len() >= size { 
            return Ok(self.drain_n(size))
        }
        while self.buffer_len() < size {
            self.read_to_buffer()?;
        }

        Ok(self.drain_n(size))
    }

    pub fn recvline(&mut self) -> Result<Vec<u8>> {
        let mut idx = 0;
        loop {
            let buffer = match self {
                Self::Tcp { buffer, .. } => buffer,
                Self::Udp { buffer, .. } => buffer,
                Self::Tls { buffer, .. } => buffer,
            };

            match buffer.iter().skip(idx).position(|&x| x == 10).map(|pos| pos + idx) {
                Some(i) => {
                    idx = i+1;
                    return Ok(self.drain_n(i+1))
                },
                None => {
                    idx = buffer.len();
                }
            }
            self.read_to_buffer()?;
        }
    }

    pub fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let suffix = suffix.as_ref();
        if suffix.len() == 0 { return Ok(vec![]); }

        let mut idx = 0;
        loop {
            let buffer = match self {
                Self::Tcp { buffer, .. } => buffer,
                Self::Udp { buffer, .. } => buffer,
                Self::Tls { buffer, .. } => buffer,
            };
            
            for j in idx..buffer.len() {
                if buffer[j] == suffix[suffix.len()-1] {
                    if suffix.len() <= buffer.len() && j >= suffix.len()-1 && suffix == &buffer[j+1-suffix.len()..j+1] {
                        return Ok(self.drain_n(j+1));
                    }
                }
            }
            idx = buffer.len();
            let _ = self.read_to_buffer()?;
        }
    }

    pub fn recvall(&mut self) -> Result<Vec<u8>> {
        while self.read_to_buffer()? != 0 {}
        return Ok(self.drain_n(self.buffer_len()));
    }

    pub fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let msg = msg.as_ref();

        if msg.len() == 0 { return Ok(()); }

        if self.is_logging() {
            eprintln!("{} {} {} ", now().red().bold(), "->".red().bold(), to_lit_colored(&msg, |x| x.normal(), |x| x.green()));
        }
        match self {
            Self::Tcp { stream, .. } => { stream.write_all(msg)?; },
            Self::Udp { stream, .. } => { stream.send(msg)?; },
            Self::Tls { stream, .. } => { stream.write_all(msg)?; },
        }
        Ok(())
    }

    pub fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let mut msg = msg.as_ref().to_vec();
        msg.push(10);
        self.send(msg)?;
        Ok(())
    }

    pub fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let out = self.recvuntil(suffix)?;
        self.sendline(msg)?;
        Ok(out)
    }

    pub fn recv_timeout(&self) -> Result<Option<Duration>> {
        return Ok(match self {
            Self::Tcp { stream, .. } => stream.read_timeout()?,
            Self::Udp { stream, .. } => stream.read_timeout()?,
            Self::Tls { stream, .. } => stream.sock.read_timeout()?,
        })
    }
    pub fn set_recv_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        match self {
            Self::Tcp { stream, .. } => stream.set_read_timeout(dur)?,
            Self::Udp { stream, .. } => stream.set_read_timeout(dur)?,
            Self::Tls { stream, .. } => stream.sock.set_read_timeout(dur)?,
        };
        Ok(())
    }

    pub fn send_timeout(&self) -> Result<Option<Duration>> {
        return Ok(match self {
            Self::Tcp { stream, .. } => stream.write_timeout()?,
            Self::Udp { stream, .. } => stream.write_timeout()?,
            Self::Tls { stream, .. } => stream.sock.write_timeout()?,
        })
    }
    pub fn set_send_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        match self {
            Self::Tcp { stream, .. } => stream.set_write_timeout(dur)?,
            Self::Udp { stream, .. } => stream.set_write_timeout(dur)?,
            Self::Tls { stream, .. } => stream.sock.set_write_timeout(dur)?,
        };
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        match self {
            Self::Tcp { stream, .. } => stream.shutdown(std::net::Shutdown::Both)?,
            Self::Udp { .. } => return Err(Error::Invalid("No shutdown for udp socket".to_string())),
            Self::Tls { stream, .. } => stream.sock.shutdown(std::net::Shutdown::Both)?,
        };
        Ok(())
    }

    fn set_nonblocking(&self, value: bool) -> Result<()> {
        return Ok(match self {
            Self::Tcp { stream, .. } => stream.set_nonblocking(value)?,
            Self::Udp { stream, .. } => stream.set_nonblocking(value)?,
            Self::Tls { stream, .. } => stream.sock.set_nonblocking(value)?,
        })
    }


    pub fn debug(&mut self) -> Result<()> {
        fn prompt() { 
            print!("{} ", "$".red());
            io::stdout().flush().expect("Unable to flush stdout");
        }
        prompt();

        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let logging = self.is_logging();
        self.log(true);

        self.set_nonblocking(true)?;

        
        let (tx, rx) = mpsc::channel();

        let stdio = std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();

            let mut buffer = [0; 1024];
            loop {
                match handle.read(&mut buffer) {
                    Ok(0) => { 
                        thread_running.store(false, Ordering::SeqCst);
                        break;
                    },
                    Ok(n) => {
                        if !thread_running.load(Ordering::SeqCst) {
                            break;
                        }
                        match from_lit(&buffer[..n]) {
                            Ok(bytes) => {
                                if let Err(e) = tx.send(bytes) {
                                    eprintln!("Unable to write to stream: {}", e);
                                }
                            },
                            Err(_e) => {},
                        }
                    },
                    Err(_e) => {
                    }
                }
            }
        });    

        loop {
            if !running.load(Ordering::SeqCst) { break; }
            match rx.try_recv() {
                Ok(data) => {
                    self.send(data)?;
                    prompt();
                },
                _ => {}
            }
            match self.recv(1024) {
                Ok(n) => {
                    if n.len() == 0 { 
                        running.store(false, Ordering::SeqCst);
                        eprint!("{}", "Pipe broke (Press Enter to continue)".red());
                        break;
                    }
                    else {
                        prompt();
                    }
                }
                
                Err(Error::Io { source }) if source.kind() == io::ErrorKind::WouldBlock => {},
                Err(e) => {
                    running.store(false, Ordering::SeqCst);
                    eprint!("{}", "Pipe broke (Press Enter to continue)".red());
                    break;
                }
            }
        }

        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        

        stdio.join().unwrap();

        self.set_nonblocking(false)?;
        self.log(logging);
        
        Ok(())
    }

    pub fn interactive(&mut self) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        self.set_nonblocking(true)?;

        
        let (tx, rx) = mpsc::channel();

        let stdio = std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();

            loop {
                if !thread_running.load(Ordering::SeqCst) {
                    break;
                }
                let mut buffer = [0; 1024];
                let n = handle.read(&mut buffer).unwrap_or(0);
                if n == 0 { 
                    thread_running.store(false, Ordering::SeqCst); 
                }
                else {
                    if let Err(e) = tx.send(buffer[..n].to_vec()) {
                        eprintln!("Unable to write to stream: {}", e);
                    }

                }
            }
        });    

        loop {
            if !running.load(Ordering::SeqCst) { break; }
            match rx.try_recv() {
                Ok(data) => {
                    self.send(data)?;
                },
                _ => {}
            }
            match self.recv(1024) {
                Ok(n) => {
                    if n.len() == 0 { 
                        running.store(false, Ordering::SeqCst);
                        eprint!("{}", "Pipe broke (Press Enter to continue)".red());
                        break;
                    }
                    else {
                        let s = String::from_utf8_lossy(&n);
                        print!("{}", s);
                    }
                }
                
                Err(Error::Io { source }) if source.kind() == io::ErrorKind::WouldBlock => {},
                Err(_e) => {
                    running.store(false, Ordering::SeqCst);
                    eprint!("{}", "Pipe broke (Press Enter to continue)".red());
                    break;
                }
            }
        }

        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        

        stdio.join().unwrap();

        self.set_nonblocking(false)?;
        
        Ok(())
    }
}

