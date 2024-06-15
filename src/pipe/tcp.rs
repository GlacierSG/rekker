use std::net::TcpStream;
use std::io::{self, Read, Write, Result, BufReader, BufRead, Error};
use std::time::Duration;
use super::pipe::Pipe;
use crate::{from_lit, to_lit_colored};
use std::sync::atomic::{AtomicBool, Ordering};
use colored::*;
use std::sync::Arc;
use regex::Regex;

pub struct Tcp {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Tcp {
    pub fn connect(addr: &str) -> std::io::Result<Tcp> {
        let re = Regex::new(r"\s+").unwrap();
        let addr = re.replace_all(addr.trim(), ":");

        let stream = TcpStream::connect(addr.as_ref())?;
        let reader = BufReader::new(stream.try_clone()?);

        Ok(Tcp{ stream, reader })
    }
}

impl Tcp {
    pub fn set_nagle(&mut self, nagle: bool) -> Result<()> {
        if nagle { self.stream.set_nodelay(false) }
        else { self.stream.set_nodelay(true) }
    }
    pub fn nagle(&self) -> Result<bool> {
        self.stream.nodelay()
    }
}

impl Pipe for Tcp {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];
        let size = self.reader.read(&mut buffer)?;
        Ok(buffer[..size].to_vec())
    }

    fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];
        let _ = self.reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn recvline(&mut self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.reader.read_until(10, &mut buffer)?;

        Ok(buffer)
    }

    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let suffix = suffix.as_ref();
        if suffix.len() == 0 {
            return Ok(vec![])
        }
        let mut buffer = vec![];

        loop {
            let mut tmp_buffer = vec![];

            let _ = self.reader.read_until(suffix[suffix.len()-1], &mut tmp_buffer)?;
            if tmp_buffer.len() == 0 {
                return Err(Error::new(io::ErrorKind::Other, "Got EOF for TCP stream"));
            }
            buffer.extend(tmp_buffer);
            if suffix.len() <= buffer.len() {
                if &suffix[..] == &buffer[(buffer.len()-suffix.len())..] {
                    return Ok(buffer);
                }
            }

        }
    }

    fn recvall(&mut self) -> Result<Vec<u8>> {
        let mut buffer = vec![];

        let _ = self.reader.read_to_end(&mut buffer).unwrap();
        Ok(buffer)
    }

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        self.stream.write_all(msg.as_ref())
    }

    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let msg = msg.as_ref();
        self.send(msg)?;
        self.send(b"\n")?;
        Ok(())
    }

    fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let buf = self.recvuntil(suffix)?;
        self.sendline(msg)?;
        Ok(buf)
    }

    fn recv_timeout(&self) -> Result<Option<Duration>> {
        self.stream.read_timeout()
    }
    fn set_recv_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.stream.set_read_timeout(dur)
    }

    fn send_timeout(&self) -> Result<Option<Duration>> {
        self.stream.write_timeout()
    }
    fn set_send_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        self.stream.set_write_timeout(dur)
    }

    fn debug(&mut self) -> Result<()> {
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


        let mut stream_clone = self.stream.try_clone()?;
        let receiver = std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                match stream_clone.read(&mut buffer) {
                    Ok(0) => {
                        println!("{}{}{}", begin_line, clear_line, "Pipe broke".red());
                        print!("{}", "Press Enter to continue".red());
                        io::stdout().flush().expect("Unable to flush stdout");

                        thread_running.store(false, Ordering::SeqCst);
                        break;
                    }, 
                    Ok(n) => {
                        let response = &buffer[0..n];
                        print!("{}{}", begin_line, clear_line);
                        let lit = to_lit_colored(&response, |x| x.normal(), |x| x.yellow());
                        
                        println!("{}",lit);
                        prompt();
                    }
                    Err(_) => {
                    }
                }

                if !thread_running.load(Ordering::SeqCst) { break; }
            }
        });    

        let stdin = io::stdin();
        let handle = stdin.lock();

        let mut bytes = vec![0; 0];
        for byte_result in handle.bytes() {
            bytes.push(byte_result?); 
            if bytes.len() != 0 && bytes[bytes.len()-1] == 10 {
                if !running.load(Ordering::SeqCst) {
                    print!("{}{}{}", go_up, begin_line, clear_line,);
                    break;
                }
                let d = from_lit(&bytes[..bytes.len()-1]);
                match d {
                    Ok(x) => {
                        bytes = x;
                        let lit = to_lit_colored(&bytes, |x| x.normal(), |x| x.green());
                        println!("{}{}{}", go_up, clear_line, lit);
                        prompt();
                        self.send(&bytes)?;
                    },
                    Err(e) => {
                        eprintln!("{}", e.red());
                        print!("{}", "$ ".red());
                        io::stdout().flush().expect("Unable to flush stdout");
                    },
                }

                bytes = vec![0; 0];
            }
        }
        print!("{}  {}", begin_line, begin_line);
        io::stdout().flush().expect("Unable to flush stdout");
        running.store(false, Ordering::SeqCst);
        
        self.set_recv_timeout(old_recv_timeout)?;

        receiver.join().unwrap();
        
        Ok(())
    }

    fn interactive(&mut self) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();


        let old_recv_timeout = self.recv_timeout()?;
        self.set_recv_timeout(Some(Duration::from_millis(1)))?;


        let mut stream_clone = self.stream.try_clone()?;
        let receiver = std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                match stream_clone.read(&mut buffer) {
                    Ok(0) => {
                        println!("{}", "Pipe broke".red());
                        print!("{}", "Press Enter to continue".red());
                        io::stdout().flush().expect("Unable to flush stdout");

                        thread_running.store(false, Ordering::SeqCst);
                        break;
                    }, 
                    Ok(n) => {
                        let response = &buffer[0..n];
                        print!("{}", String::from_utf8_lossy(&response));
                        io::stdout().flush().expect("Unable to flush stdout");
                    }
                    Err(_) => {}
                }

                if !thread_running.load(Ordering::SeqCst) { break; }
            }
        });    

        let stdin = io::stdin();
        let handle = stdin.lock();

        let mut bytes = vec![0; 0];
        for byte_result in handle.bytes() {
            bytes.push(byte_result?);
            if bytes[bytes.len()-1] == 10 {
                if !running.load(Ordering::SeqCst) {
                    break;
                }
    
                self.send(&bytes)?;

                bytes = vec![0; 0];
            }
        }
        running.store(false, Ordering::SeqCst);
        
        self.set_recv_timeout(old_recv_timeout)?;

        receiver.join().unwrap();
        
        Ok(())
    }


    fn close(&mut self) -> Result<()> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}

