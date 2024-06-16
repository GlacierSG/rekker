use std::io::Result;
use std::time::Duration;

pub trait Pipe {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>>;
    fn recvn(&mut self, size: usize) -> Result<Vec<u8>>;
    fn recvline(&mut self) -> Result<Vec<u8>>;
    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>>;
    fn recvall(&mut self) -> Result<Vec<u8>>;

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<()>;
    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<()>;
    fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>>;
    
    fn set_recv_timeout(&mut self, dur: Option<Duration>) -> Result<()>;
    fn recv_timeout(&self) -> Result<Option<Duration>>;

    fn set_send_timeout(&mut self, dur: Option<Duration>) -> Result<()>;
    fn send_timeout(&self) -> Result<Option<Duration>>;

    fn close(&mut self) -> Result<()>;
}

