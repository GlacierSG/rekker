use std::net::UdpSocket;
use std::io::{self, Read, Write, Result, Error};
use std::time::Duration;
use super::pipe::Pipe;
use crate::{from_lit, to_lit_colored};
use std::sync::atomic::{AtomicBool, Ordering};
use colored::*;
use std::sync::Arc;
use std::mem;
use std::io::ErrorKind;
use regex::Regex;

pub struct Udp {
    stream: Option<UdpSocket>,
    buffer: Vec<u8>,
}

macro_rules! handle_stream_option {
    ($stream:expr) => {
        match $stream {
            Some(stream) => stream,
            None => return Err(Error::new(io::ErrorKind::Other, "UDP Socket is not connected")),
        }
    }
}
impl Udp {
    pub fn listen(addr: &str) -> std::io::Result<Udp> {
        let re = Regex::new(r"\s+").unwrap();
        let addr = re.replace_all(addr.trim(), ":");

        let stream = UdpSocket::bind(addr.as_ref())?; 

        Ok(Udp {
            stream: Some(stream),
            buffer: vec![0; 0]
        })
    }

    pub fn connect(addr: &str) -> std::io::Result<Udp> {
        let re = Regex::new(r"\s+").unwrap();
        let addr = re.replace_all(addr.trim(), ":");

        let stream = UdpSocket::bind("0.0.0.0:0")?; 
        stream.connect(addr.as_ref())?;

        Ok(Udp {
            stream: Some(stream),
            buffer: vec![0; 0]
        })
    }

    fn add_to_buffer(&mut self) -> Result<usize> {
        handle_stream_option!(&self.stream).set_nonblocking(true)?;
        let mut total = 0;
        loop {
            match self.add_to_buffer_blocking() {
                Ok(r) => {
                    total += r;
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        break
                    }
                    else {
                        handle_stream_option!(&self.stream).set_nonblocking(false)?;
                        return Err(e);
                    }
                }
            }
        }
        handle_stream_option!(&self.stream).set_nonblocking(false)?;

        Ok(total)
    }

    fn add_to_buffer_blocking(&mut self) -> Result<usize> {
        let mut temp_buf = vec![0; 65535];

        match handle_stream_option!(&self.stream).recv(&mut temp_buf) {
            Ok(r) => {
                self.buffer.extend_from_slice(&temp_buf[..r]);
                Ok(r)
            },
            Err(e) => Err(e)
        }
    }
}

impl Pipe for Udp {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        self.add_to_buffer()?;
       
        let l = if size < self.buffer.len() { size }
                else { self.buffer.len() };

        let out = self.buffer[..l].to_vec();
        self.buffer.drain(..l);
        Ok(out)
    }
    fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        while self.buffer.len() < size {
            self.add_to_buffer()?;
        }

        let out = self.buffer[..size].to_vec();
        self.buffer.drain(..size);
        Ok(out)
    }
    fn recvline(&mut self) -> Result<Vec<u8>> {
        let mut bi = 0;
        loop {
            for i in bi..self.buffer.len() {
                if self.buffer[i] == 10 {
                    let out = self.buffer[..i+1].to_vec();
                    self.buffer.drain(..i+1);
                    return Ok(out);
                }
            }
            bi = self.buffer.len();

            self.add_to_buffer_blocking()?;
        }
    }
    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let suffix = suffix.as_ref();
        if suffix.len() == 0 {
            return Ok(vec![])
        }

        let mut bi = suffix.len() - 1;
        loop {
            if suffix.len() <= self.buffer.len() {
                for i in bi..self.buffer.len() {
                    if self.buffer[i] == suffix[suffix.len()-1] {
                        if &suffix[..] == &self.buffer[i+1-suffix.len()..i+1] {
                            let out = self.buffer[..i+1].to_vec();
                            self.buffer.drain(..i+1);
                            return Ok(out);
                        }
                    }
                }
                bi = self.buffer.len();
            }

            self.add_to_buffer_blocking()?;
        }
    }
    fn recvall(&mut self) -> Result<Vec<u8>> {
        self.add_to_buffer()?;
        Ok(mem::take(&mut self.buffer))
    }

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let msg = msg.as_ref();
        let mut total = 0;
        while total < msg.len() {
            total += handle_stream_option!(&self.stream).send(&msg[total..])?;
        }
        Ok(())
    }
    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let msg = msg.as_ref();
        let _ = self.send(msg);
        let _ = self.send(b"\n");
        Ok(())
    }
    fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let buf = self.recvuntil(suffix)?;
        self.sendline(msg)?;
        Ok(buf)
    }

    fn recv_timeout(&self) -> Result<Option<Duration>> {
        handle_stream_option!(&self.stream).read_timeout()
    }
    fn set_recv_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        handle_stream_option!(&self.stream).set_read_timeout(dur)
    }

    fn send_timeout(&self) -> Result<Option<Duration>> {
        handle_stream_option!(&self.stream).write_timeout()
    }
    fn set_send_timeout(&mut self, dur: Option<Duration>) -> Result<()> {
        handle_stream_option!(&self.stream).set_write_timeout(dur)
    }

    fn close(&mut self) -> Result<()> {
        self.stream = None;
        self.buffer.clear();
        Ok(())
    }
}

impl Udp {
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


        let stream_clone = handle_stream_option!(&self.stream).try_clone()?;
        let receiver = std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                match stream_clone.recv(&mut buffer) {
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

    pub fn interactive(&mut self) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();


        let old_recv_timeout = self.recv_timeout()?;
        self.set_recv_timeout(Some(Duration::from_millis(1)))?;


        let stream_clone = handle_stream_option!(&self.stream).try_clone()?;
        let receiver = std::thread::spawn(move || {
            let mut buffer = [0; 1024];
            loop {
                match stream_clone.recv(&mut buffer) {
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

}
