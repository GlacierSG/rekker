use std::net::TcpStream;
use std::io::{self, Read, Write, Result, BufReader, BufRead};
use std::time::Duration;
use super::pipe::Pipe;
use crate::{from_lit, to_lit_colored};
use std::sync::atomic::{AtomicBool, Ordering};
use colored::*;
use std::sync::Arc;

pub struct Tcp {
    pub stream: TcpStream
}

impl Tcp {
    pub fn connect(addr: &str) -> std::io::Result<Tcp> {
        Ok(Tcp {
            stream: TcpStream::connect(addr)?
        })
    }

}

impl Pipe for Tcp {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];
        let size = self.stream.read(&mut buffer)?;
        Ok(buffer[..size].to_vec())
    }

    fn recvn(&mut self, size: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; size];
        let _ = self.stream.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn recvline(&mut self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(&self.stream);

        reader.read_until(10, &mut buffer)?;

        Ok(buffer)
    }

    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let suffix = suffix.as_ref();
        if suffix.len() == 0 {
            return Ok(vec![])
        }
        let mut buffer = vec![];

        let mut reader = BufReader::new(&self.stream);
        loop {
            let mut tmp_buffer = vec![];

            let _ = reader.read_until(suffix[suffix.len()-1], &mut tmp_buffer)?;
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

        let mut reader = BufReader::new(&self.stream);
        let _ = reader.read_to_end(&mut buffer).unwrap();
        Ok(buffer)
    }

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<usize> {
        self.stream.write(msg.as_ref())
    }

    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<usize> {
        let mut tmp: Vec<u8> = msg.as_ref().to_vec();
        tmp.extend(b"\n");
        self.send(tmp.as_slice())
    }

    fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>> {
        let buf = self.recvuntil(suffix)?;
        self.sendline(msg)?;
        Ok(buf)
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


        let old_read_timeout = self.stream.read_timeout()?;
        self.stream.set_read_timeout(Some(Duration::from_millis(1)))?;


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
                    print!("{}{}{}", go_up, begin_line, clear_line,);
                    break;
                }
                let d;
                if bytes.len() > 0 {
                    d = from_lit(&bytes[..bytes.len()-1]);
                }
                else {
                    d = from_lit(&bytes);
                }
                match d {
                    Ok(x) => {
                        bytes = x;
                        let lit = to_lit_colored(&bytes, |x| x.normal(), |x| x.green());
            
                        println!("{}{}{}", go_up, clear_line, lit);
                        prompt();
                        self.stream.write_all(&bytes)?;
                        self.stream.flush()?;
                    },
                    Err(e) => eprintln!("{}", e.red()),
                }

                bytes = vec![0; 0];
            }
        }
        running.store(false, Ordering::SeqCst);
        
        self.stream.set_read_timeout(old_read_timeout)?;

        receiver.join().unwrap();
        
        Ok(())
    }

    fn interactive(&mut self) -> Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();


        let old_read_timeout = self.stream.read_timeout()?;
        self.stream.set_read_timeout(Some(Duration::from_millis(1)))?;


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
    
                //print!("{}", String::from_utf8_lossy(&bytes));
                //io::stdout().flush().expect("Unable to flush stdout");

                self.stream.write_all(&bytes)?;
                self.stream.flush()?;

                bytes = vec![0; 0];
            }
        }
        running.store(false, Ordering::SeqCst);
        
        self.stream.set_read_timeout(old_read_timeout)?;

        receiver.join().unwrap();
        
        Ok(())
    }


    fn close(&mut self) -> Result<()> {
        self.stream.shutdown(std::net::Shutdown::Both)
    }
}

