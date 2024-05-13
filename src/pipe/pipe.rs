use std::io::Result;

pub trait Pipe {
    fn recv(&mut self, size: usize) -> Result<Vec<u8>>;
    fn recvn(&mut self, size: usize) -> Result<Vec<u8>>;
    fn recvline(&mut self) -> Result<Vec<u8>>;
    fn recvuntil(&mut self, suffix: impl AsRef<[u8]>) -> Result<Vec<u8>>;
    fn recvall(&mut self) -> Result<Vec<u8>>;

    fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<usize>;
    fn sendline(&mut self, msg: impl AsRef<[u8]>) -> Result<usize>;
    fn sendlineafter(&mut self, suffix: impl AsRef<[u8]>, msg: impl AsRef<[u8]>) -> Result<Vec<u8>>;

    fn debug(&mut self) -> Result<()>;
    fn interactive(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

