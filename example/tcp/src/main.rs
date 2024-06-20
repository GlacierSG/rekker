use rekker::*;
use std::io::stdout;
use std::io::Write;

fn main() {
    let mut io = Tcp::connect("localhost 9000").unwrap();
    io.log(true);

    //dbg!(io.recv(3).unwrap());
    //dbg!(io.recvuntil(b"abc"));
    /*
    dbg!(io.recvuntil("abc").unwrap());
    io.recvline();
    io.sendline("laksjdlkjasdlækj");
*/
    io.debug();
}
