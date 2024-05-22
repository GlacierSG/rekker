use rekker::Req;

fn main() -> () {
    let mut req = Req::new()
        .url(b"https://test.com/abc")
        .header(b"abc", b"d\nef")
        .header(b"abc", b"def")
        .header(b"abc", b"def")
        .header(b"abc", b"def")
        .body(b"asdf");
    println!("{}", req.pretty());

    let mut req = Req::new()
        .url("/abc")
        .header(b"abc", b"def")
        .body(b"asdf");
    println!("{}", req.pretty());

}
