use rekker::Req;

fn main() -> () {
    let mut req = Req::new()
        .get("https://test.com/abc")
        .header(b"abc", b"d\nef")
        .header(b"abc", b"def")
        .header(b"abc", b"def")
        .header(b"abc", b"def")
        .proxy("localhost:8080")
        .data(b"asdf");
    println!("{}", req.to_string());

    let mut req = Req::from_string(r"
GET /abc HTTP/1.1
host: localhost:8080
test: abc
lksajdlkjsad: alksjdlksad

asldkjsalkdj").expect("Could not parse http");

    println!("{}", req.to_string());
}
