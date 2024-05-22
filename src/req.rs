use crate::literal::to_lit_colored;
use colored::*;

pub struct Req {
    pub method: Vec<u8>,
    pub path: Vec<u8>,
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub body_raw: Vec<u8>,
    pub proxied: bool,
    pub host: Vec<u8>,
    pub tls: bool,
}

impl Req {
    pub fn new() -> Req {
        Req {
            method: b"GET".to_vec(),
            path: b"/".to_vec(),
            headers: vec![],
            body_raw: b"".to_vec(),
            host: b"".to_vec(),
            proxied: false,
            tls: false,
        }
    }

    pub fn url(mut self, value: impl AsRef<[u8]>) -> Self {
        let value = value.as_ref();

        let mut t = 0;
        if value.len() >= 8 && &value[..8] == b"https://" {
            self.tls = true;
            t = 8;
        }
        else if value.len() >= 7 && &value[..7] == b"http://" {
            self.tls = false;
            t = 7;
        }

        let mut l = value.len();
        for i in t..value.len() {
            if value[i] == 47 {
                l = i;
                break;
            }
        }
        self.host = value[..l].to_vec();
        self.path = value[l..].to_vec();
        if l-t > 1 {
            self = self.header(b"Host", &value[t..l].to_vec());
        }
        self
    }

    pub fn header(mut self, header: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Self {
        self.headers.push((header.as_ref().to_vec(), value.as_ref().to_vec()));
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref();
        self.body_raw = body.to_vec();
        self
    }

    pub fn pretty(&self) -> String {
        fn colored(b: &[u8]) -> String {
            to_lit_colored(b, |x| x.into(), |x| x.yellow())
        }
        let mut out = colored(&self.method);
        out.push_str(" ");
        if self.proxied {
            out.push_str(&colored(&self.host));
        }
        out.push_str(&colored(&self.path));
        out.push_str(" HTTP/1.1\n");
        for (header, value) in &self.headers {
            out.push_str(&colored(&header));
            out.push_str(": ");
            out.push_str(&colored(&value));
            out.push_str("\n");
        }
        out.push_str("\n");
        out.push_str(&colored(&self.body_raw));
        out
    }

    pub fn proxy(mut self) -> Self {
        self.proxied = true;
        self
    }
}
