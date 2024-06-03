use std::io::{Read, Write, Result};
use std::net::TcpStream;
use std::sync::Arc;
use regex::Regex;
use colored::Colorize;
use super::pipe::Pipe;
use std::io;
use crate::{to_lit_colored, from_lit};
use std::sync::atomic::{AtomicBool,Ordering};
use rustls::RootCertStore;
use std::io::{ErrorKind, Error};
use std::time::Duration;

pub struct Tls {
    stream: TcpStream,
    conn: rustls::ClientConnection,
}

impl Tls {
    pub fn connect(addr: &str) -> Result<Tls> {
        let re = match Regex::new(r"\s+") {
            Ok(r) => r,
            Err(e) => return Err(Error::new(ErrorKind::Other, format!("{}", e)))};

        let addr: String = re.replace_all(addr.trim(), ":").into_owned();

        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Allow SSLKEYLOGFILE
        config.key_log = Arc::new(rustls::KeyLogFile::new());
        
        let stream = TcpStream::connect(&addr)?;
        let t1: Vec<&str> = addr.split(|b| b as u32 == 58).collect();
        if let Some(t1) = t1.get(0) {
            if let Ok(t2) = (*t1).to_string().try_into() {
                let conn = rustls::ClientConnection::new(Arc::new(config), t2).unwrap();

                return Ok(Tls {
                    stream: stream,
                    conn: conn,
                });
            }
        }
        return Err(Error::new(ErrorKind::Other, "Bad TLS address format"));
    }
}

impl Tls {
    pub fn set_nagle(&mut self, nagle: bool) -> Result<()> {
        self.stream.set_nodelay(nagle)
    }
    pub fn nagle(&self) -> Result<bool> {
        self.stream.nodelay()
    }
}

impl Pipe for Tls {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        let mut buffer = vec![0; size];
        tls.read(&mut buffer)?;
        Ok(buffer)
    }
    fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        let mut buffer = vec![0; size];
        let mut total = 0;
        while total >= size {
            total += tls.read(&mut buffer[total..])?;
        }
        Ok(buffer)
    }
    fn recvline(&mut self) -> Result<Vec<u8>> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        let mut buffer = vec![];
        while buffer.len() == 0 || buffer[buffer.len()-1] != 10 {
            let mut byte = vec![0; 1];
            let l = tls.read(&mut byte)?;
            if l == 1 {
                buffer.extend(&byte);
            }
        }

        Ok(buffer)
    }
    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let suffix = suffix.as_ref();
        if suffix.len() == 0 {
            return Ok(vec![]);
        }
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        let mut buffer = vec![];
        loop {
            while buffer.len() == 0 || buffer[buffer.len()-1] != suffix[suffix.len()-1] {
                let mut byte = vec![0; 1];
                let l = tls.read(&mut byte)?;
                if l == 1 {
                    buffer.extend(&byte);
                }
            }
            if buffer.len() >= suffix.len() && &suffix[..] == &buffer[suffix.len()-suffix.len()..] {
                return Ok(buffer);
            }
        }
    }
    fn recvall(&mut self) -> Result<Vec<u8>> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        let mut buffer = Vec::new();
        tls.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        
        let msg = msg.as_ref();
        let _ = tls.write(msg);
        Ok(())
    }
    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        
        let msg = msg.as_ref();
        let _ = tls.write(msg);
        let _ = tls.write(b"\n");
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

