use rekker::*;
use std::io::stdout;
use std::io::Write;

fn main() {
    let mut io = Tcp::connect("localhost 9000").unwrap();
    io.log(true);
    dbg!(io.recvuntil("abc"));
    io.recvline();
    io.sendline("laksjdlkjasdlækj");
    io.interactive();
}
