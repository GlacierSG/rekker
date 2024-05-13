use std::io::{Read, Write, Result};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::RootCertStore;


pub struct Tls {
    stream: TcpStream,
    conn: rustls::ClientConnection,
}

impl Tls {
    pub fn connect() -> Tls {
        let root_store = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // Allow SSLKEYLOGFILE
        config.key_log = Arc::new(rustls::KeyLogFile::new());
        
        let stream = TcpStream::connect("www.rust-lang.org:443").unwrap();
        let server_name = "www.rust-lang.org".try_into().unwrap();
        let conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();

        Tls {
            stream: stream,
            conn: conn,
        }
    }

    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        let mut plaintext = Vec::new();
        tls.read_to_end(&mut plaintext)?;
        Ok(plaintext)
    }

    pub fn send(&mut self, msg: impl AsRef<[u8]>) -> Result<usize> {
        let mut tls = rustls::Stream::new(&mut self.conn, &mut self.stream);
        
        let msg = msg.as_ref();
        tls.write(msg)
    }
}

