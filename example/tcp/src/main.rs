use rekker::*;
use std::io::stdout;
use std::io::Write;

fn main() {
/*
    //let syndis = Dns::new("syndis.is");
    let mut io = Tcp::new("localhost:1234").unwrap();
    let req = Req::new()
        .url("http://syndis.is/abcabcabc")
        .header(b"someheader", b"abc")
        .body(b"abc")
        .proxy()
        .body(b"slkdjf");

    io.send(req.raw());
    io.close();
    */
    /*
    //dbg!(req.raw());
    let mut tls = Tls::connect();
    //test()
   tls.send(
        concat!(
            "GET / HTTP/1.1\r\n",
            "Host: www.rust-lang.org\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        )
        .as_bytes()
    );
   stdout().write_all(&tls.recv().unwrap()).unwrap();
*/

    let mut io = Tcp::connect("localhost 9000").unwrap();
    println!("{:?}",io.recvuntil(b"abc").unwrap());
    io.debug();
    io.interactive();
}
