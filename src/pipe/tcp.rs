use std::net::TcpStream;
use std::io::{self, Read, Write, Result, BufReader, BufRead, Error};
use std::time::Duration;
use super::pipe::Pipe;
use crate::{from_lit, to_lit_colored};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use colored::*;
use regex::Regex;
use super::buffer::Buffer;

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
        self.buffer.logging_on = logging;
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

        let old_recv_timeout = self.recv_timeout()?;
        self.set_recv_timeout(Some(Duration::from_millis(1)))?;


        let mut stream_clone = self.buffer.stream.try_clone()?;
        let receiver = std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();

            let mut buffer = [0; 65535];
            loop {
                match handle.read(&mut buffer) {
                    Ok(0) => { 
                        thread_running.store(false, Ordering::SeqCst);
                        print!("{}{}", begin_line, clear_line,);
                        io::stdout().flush().expect("Unable to flush stdout");
                        break;
                    },
                    Ok(n) => {
                        if !thread_running.load(Ordering::SeqCst) {
                            print!("{}{}{}", go_up, begin_line, clear_line,);
                            io::stdout().flush().expect("Unable to flush stdout");
                            break;
                        }
                        match from_lit(&buffer[..n-1]) {
                            Ok(bytes) => {
                                let lit = to_lit_colored(&bytes, |x| x.normal(), |x| x.green());
                                println!("{}{}{} {}", go_up, clear_line, "->".red().bold(), lit);
                                prompt();
                                if let Err(e) = stream_clone.write_all(&bytes) {
                                    eprintln!("Unable to write to stream: {}", e);
                                }
                            },
                            Err(e) => {
                                eprintln!("{}", e.red());
                                print!("{}", "$ ".red());
                                io::stdout().flush().expect("Unable to flush stdout");
                            },
                        }
                    },
                    Err(_e) => {
                    }
                }
            }
        });    



        let mut buffer = [0; 1024];
        loop {
            match self.buffer.stream.read(&mut buffer) {
                Ok(0) => {
                    println!("{}{}{}", begin_line, clear_line, "Pipe broke".red());
                    print!("{}", "Press Enter to continue".red());
                    io::stdout().flush().expect("Unable to flush stdout");

                    running.store(false, Ordering::SeqCst);
                    break;
                }, 
                Ok(n) => {
                    let response = &buffer[0..n];
                    print!("{}{}", begin_line, clear_line);
                    let lit = to_lit_colored(&response, |x| x.normal(), |x| x.yellow());
                    
                    println!("{} {}", "<-".red().bold(), lit);
                    prompt();
                }
                Err(_) => {
                }
            }

            if !running.load(Ordering::SeqCst) { break; }
        }



        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        
        self.set_recv_timeout(old_recv_timeout)?;

        receiver.join().unwrap();
        
        Ok(())
    }


    pub fn interactive(&mut self) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let old_recv_timeout = self.recv_timeout()?;
        self.set_recv_timeout(Some(Duration::from_millis(1)))?;


        let mut stream_clone = self.buffer.stream.try_clone()?;
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
                                if let Err(e) = stream_clone.write_all(&bytes) {
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



        let mut buffer = [0; 1024];
        loop {
            match self.buffer.stream.read(&mut buffer) {
                Ok(0) => {
                    running.store(false, Ordering::SeqCst);
                    break;
                }, 
                Ok(n) => {
                    let response = &buffer[0..n];
                    print!("{}", String::from_utf8_lossy(&response));
                    io::stdout().flush().expect("Unable to flush stdout");
                }
                Err(_) => {
                }
            }

            if !running.load(Ordering::SeqCst) { break; }
        }



        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        
        self.set_recv_timeout(old_recv_timeout)?;

        receiver.join().unwrap();
        
        Ok(())
    }
}

