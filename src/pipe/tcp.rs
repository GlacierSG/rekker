use std::net::TcpStream;
use std::io::{self, Read, Write, Result, BufReader, BufRead, Error};
use std::time::Duration;
use super::pipe::Pipe;
use crate::{from_lit, to_lit_colored};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use colored::*;
use regex::Regex;
use super::buffer::{Buffer, WriteBuffer};

#[derive(Debug)]
pub struct Tcp {
    buffer: Buffer<TcpStream>,
}

impl Tcp {
    pub fn connect(addr: &str) -> Result<Tcp> {
        let re = Regex::new(r"\s+").unwrap();
        let addr = re.replace_all(addr.trim(), ":");

        let stream = TcpStream::connect(addr.as_ref())?;
        let buffer = Buffer::new(stream);
    
        let mut tcp = Tcp{ buffer };
        let _ = tcp.set_nagle(false)?;

        Ok(tcp)
    }
    
    pub fn from_stream(stream: TcpStream) -> Result<Self> {
        let buffer = Buffer::new(stream);
        Ok(Tcp { buffer })
    }
}

impl Tcp {
    pub fn log(&mut self, logging: bool) {
        self.buffer.logging = logging;
    }
    pub fn set_nagle(&mut self, nagle: bool) -> Result<()> {
        self.buffer.stream.set_nodelay(!nagle)
    }
    pub fn nagle(&self) -> Result<bool> {
        Ok(!(self.buffer.stream.nodelay()?))
    }
}

impl Pipe for Tcp {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        self.buffer.recv(size)
    }

    fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        self.buffer.recvn(size)
    }

    fn recvline(&mut self) -> Result<Vec<u8>> {
        self.buffer.recvline()
    }

    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        self.buffer.recvuntil(suffix)
    }

    fn recvall(&mut self) -> Result<Vec<u8>> {
        self.buffer.recvall()
    }

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        self.buffer.send(msg)
    }

    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        self.buffer.sendline(msg)
    }

    fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        self.buffer.sendlineafter(suffix, msg)
    }

    fn recv_timeout(&self) -> Result<Option<Duration>> {
        self.buffer.stream.read_timeout()
    }
    fn set_recv_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.buffer.stream.set_read_timeout(dur)
    }

    fn send_timeout(&self) -> Result<Option<Duration>> {
        self.buffer.stream.write_timeout()
    }
    fn set_send_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.buffer.stream.set_write_timeout(dur)
    }

    fn close(&mut self) -> Result<()> {
        self.buffer.stream.shutdown(std::net::Shutdown::Both)
    }
}
impl Tcp {
    pub fn debug(&mut self) -> Result<()> {
        let go_up = "\x1b[1A";
        let clear_line = "\x1b[2K";
        let begin_line = "\r";
        fn prompt() { 
            print!("{} ", "$".red());
            io::stdout().flush().expect("Unable to flush stdout");
        }
        prompt();

        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let logging = self.buffer.logging;
        self.buffer.logging = true;

        self.buffer.stream.set_nonblocking(true)?;


        let mut writer = WriteBuffer::from_stream(self.buffer.stream.try_clone()?, self.buffer.logging);
        let receiver = std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();

            let mut buffer = [0; 65535];
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
                                let lit = to_lit_colored(&bytes, |x| x.normal(), |x| x.green());
                                if let Err(e) = writer.send(&bytes) {
                                    eprintln!("Unable to write to stream: {}", e);
                                }
                                prompt();
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
            match self.buffer.recv(1024) {
                Ok(buffer) => {
                    prompt();
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {},
                Err(e) => {
                    running.store(false, Ordering::SeqCst);
                    eprintln!("{}", format!("{}", e).red());
                    print!("{}", "Press Enter to continue".red());
                }
            }

            if !running.load(Ordering::SeqCst) { break; }
        }

        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        

        receiver.join().unwrap();

        self.buffer.stream.set_nonblocking(false)?;
        self.buffer.logging = logging;
        
        Ok(())
    }

    pub fn interactive(&mut self) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();
        
        self.buffer.stream.set_nonblocking(true)?;


        let mut writer = WriteBuffer::from_stream(self.buffer.stream.try_clone()?, self.buffer.logging);
        let receiver = std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();

            let mut buffer = [0; 65535];
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
                        if let Err(e) = writer.send(&buffer[..n]) {
                            eprintln!("Unable to write to stream: {}", e);
                        }
                    },
                    Err(_e) => {}
                }
            }
        });    



        loop {
            match self.buffer.recv(1024) {
                Ok(buffer) => {
                    print!("{}", String::from_utf8_lossy(&buffer));
                    io::stdout().flush().expect("Unable to flush stdout");
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {},
                Err(e) => {
                    running.store(false, Ordering::SeqCst);
                    eprintln!("{}", format!("{}", e).red());
                    print!("{}", "Press Enter to continue".red());
                }
            }

            if !running.load(Ordering::SeqCst) { break; }
        }



        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        
        self.buffer.stream.set_nonblocking(false)?;

        receiver.join().unwrap();
        
        Ok(())
    }
}

