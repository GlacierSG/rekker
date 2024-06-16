use crate::literal::to_lit_colored;
use colored::*;

pub struct Req {
    pub raw_method: Vec<u8>,
    pub raw_headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub raw_body: Vec<u8>,
    pub raw_path: Vec<u8>,
    pub raw_url: Vec<u8>,
    pub raw_proxy: String,
    pub is_tls: bool,
}

impl Req {
    pub fn new() -> Req {
        Req {
            raw_method: b"GET".to_vec(),
            raw_path: b"/".to_vec(),
            raw_url: b"".to_vec(),
            raw_headers: vec![],
            raw_body: b"".to_vec(),
            raw_proxy: String::new(),
            is_tls: false,
        }
    }

    fn url(mut self, url: impl AsRef<[u8]>) -> Self {
        let url = url.as_ref();

        let mut t = 0;
        if url.len() >= 8 && &url[..8] == b"https://" {
            self.is_tls = true;
            t = 8;
        }
        else if url.len() >= 7 && &url[..7] == b"http://" {
            self.is_tls = false;
            t = 7;
        }

        let mut l = url.len();
        for i in t..url.len() {
            if url[i] == 47 {
                l = i;
                break;
            }
        }
        self.raw_url = url[..l].to_vec();
        self.raw_path = url[l..].to_vec();
        if l-t >= 1 {
            return self.header(b"Host", &url[t..l].to_vec());
        }
        self
    }
    pub fn get(self, url: impl AsRef<[u8]>) -> Self {
        self.method(b"GET")
            .url(url.as_ref())
    }
    pub fn post(self, url: impl AsRef<[u8]>) -> Self {
        self.method(b"POST")
            .url(url.as_ref())
    }
    pub fn put(self, url: impl AsRef<[u8]>) -> Self {
        self.method(b"PUT")
            .url(url.as_ref())
    }
    pub fn method(mut self, method: impl AsRef<[u8]>) -> Self {
        self.raw_method = method.as_ref().to_vec();
        self
    }

    pub fn path(mut self, path: impl AsRef<[u8]>) -> Self {
        self.raw_path = path.as_ref().to_vec();
        self
    }

    pub fn header(mut self, header: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Self {
        self.raw_headers.push((header.as_ref().to_vec(), value.as_ref().to_vec()));
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref();
        self.raw_body = body.to_vec();
        self
    }
    pub fn data(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref();
        self.raw_body = body.to_vec();
        self.header(b"Content-Length", body.len().to_string())
    }

    pub fn to_string(&self) -> String {
        fn colored(b: &[u8]) -> String {
            to_lit_colored(b, |x| x.into(), |x| x.yellow())
        }
        let mut out = colored(&self.raw_method);
        out.push_str(" ");
        if self.raw_proxy.len() != 0 {
            out.push_str(&colored(&self.raw_url));
        }
        out.push_str(&colored(&self.raw_path));
        out.push_str(" HTTP/1.1\n");
        for (header, value) in &self.raw_headers {
            out.push_str(&colored(&header));
            out.push_str(": ");
            out.push_str(&colored(&value));
            out.push_str("\n");
        }
        out.push_str("\n");
        out.push_str(&colored(&self.raw_body));
        out
    }

    pub fn from_string(value: &str) -> Result<Req, ()> {
        let mut req = Req::new();
        let mut total = 0;
        let mut lines;
        if let Some(v) = value.strip_prefix("\n") {
            lines = v.split("\n");
        }
        else {
            return Err(());
        }
        if let Some(first_line) = lines.next() {
            total += first_line.len()+1;
            let mut parts = first_line.splitn(2, " ");
            if let Some(method) = parts.next() {
                req.raw_method = method.as_bytes().to_vec();
            } else {
                return Err(());
            }
            if let Some(r) = parts.next() {
                let mut l = 0;
                if r.ends_with("HTTP/1.1") {
                    l = 8;
                }
                else if r.ends_with("HTTP/2") {
                    l = 6;
                }
                else {
                    return Err(());
                }
                if r.len() > l {
                    req.raw_path = r[..r.len()-l-1].as_bytes().to_vec();
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
        }
        else {
            return Err(());
        }
        loop {
            if let Some(line) = lines.next() {
                total += line.len()+1;
                if line.len() == 0 { break; }
                let mut h = line.splitn(2, ": ");
                let header = h.next();
                let value = h.next();
                if let Some(header) = header{
                    if let Some(value) = value {
                        req = req.header(header, value);
                    }
                    else {
                        return Err(());
                    }
                }
                else {
                    return Err(());
                }
            }
            else {
                return Err(());
            }
        }
        if value.len() < total+1 {
            return Err(());
        }
        req.raw_body = value[total+1..].as_bytes().to_vec();
        Ok(req)
    }

    pub fn proxy(mut self, proxy: &str) -> Self {
        self.raw_proxy = proxy.to_string();
        self
    }
}
