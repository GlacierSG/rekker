use rekker::*;
use std::io::stdout;
use std::io::Write;

fn main() {
    let mut io = Pipe::tcp("localhost 9000").unwrap();
    //io.log(true);
    io.interactive();
    return;
    //dbg!(io.recv(3).unwrap());
    //dbg!(io.recvuntil(b"abc"));
    dbg!(io.recvuntil("abc\n").unwrap());
    io.recvline();
    io.sendline("laksjdlkjasdlækj");
}
