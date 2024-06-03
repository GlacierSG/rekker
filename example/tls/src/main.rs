use rekker::*;
use std::io::stdout;
use std::io::Write;

fn main() -> std::io::Result<()>{
    let mut io = Tls::connect("localhost:6666")?;
    io.send(b"abc")?;
    let a = [0,1];
    println!("{:?}", &a[..2]);
    println!("{:?}",io.recv(1)?);
    println!("{:?}",io.recv(2)?);
    //io.debug()?;
    //io.interactive()?;
    Ok(())
}
