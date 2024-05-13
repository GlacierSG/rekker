use std::net::UdpSocket;

struct Udp {
    stream: UdpSocket,
}

impl Udp {
    pub fn connect(addr: &str) -> std::io::Result<Udp> {
        Ok(Udp {
            stream: UdpSocket::bind(addr)?
        })
    }
}
