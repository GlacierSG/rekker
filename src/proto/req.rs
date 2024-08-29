use crate::literal::to_lit_colored;
use crate::{Res, Tcp, Tls, Error, Result, Pipe, Http1};
use colored::*;

pub struct Req {
    pub raw_method: Vec<u8>,
    pub raw_path: Vec<u8>,
    pub raw_headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub raw_body: Vec<u8>,
    pub raw_url: Vec<u8>,
    pub is_proxy: bool,
    pub is_tls: bool,
}

impl Req {
    pub fn new() -> Self {
        Self {
            raw_method: vec![],
            raw_path: vec![],
            raw_url: vec![],
            raw_headers: vec![],
            raw_body: vec![],
            is_proxy: false,
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
            if url[i] == 47 { // Find next `/`
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
    pub fn get(url: impl AsRef<[u8]>) -> Self {
        Self::new()
            .method(b"GET")
            .url(url)
    }
    pub fn post(url: impl AsRef<[u8]>) -> Self {
        Self::new()
            .method(b"POST")
            .url(url)
    }
    pub fn put(url: impl AsRef<[u8]>) -> Self {
        Self::new()
            .method(b"PUT")
            .url(url)
    }
    pub fn delete(url: impl AsRef<[u8]>) -> Self {
        Self::new()
            .method(b"DELETE")
            .url(url)
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
        if self.is_proxy {
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

    pub fn from_string(value: &str) -> Result<Req> {
        let mut req = Req::new();
        let mut total = 0;
        let mut lines = value.strip_prefix("\n").unwrap_or(value).split("\n");

        if let Some(first_line) = lines.next() {
            total += first_line.len()+1;
            let mut parts = first_line.splitn(2, " ");
            if let Some(method) = parts.next() {
                req.raw_method = method.as_bytes().to_vec();
            } else {
                return Err(Error::ParsingError("no method".to_string()));
            }
            if let Some(r) = parts.next() {
                let l;
                if r.ends_with("HTTP/1.1") {
                    l = 8;
                }
                else if r.ends_with("HTTP/2") {
                    l = 6;
                }
                else {
                    return Err(Error::ParsingError("unknown protocol".to_string()));
                }
                if r.len() > l {
                    req.raw_path = r[..r.len()-l-1].as_bytes().to_vec();
                } else {
                    return Err(Error::ParsingError("no path".to_string()));
                }
            } else {
                return Err(Error::ParsingError("unable to parse after method".to_string()));
            }
        }
        else {
            return Err(Error::ParsingError("unable to parse first line".to_string()));
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
                        return Err(Error::ParsingError("unable to parse header value".to_string()));
                    }
                }
                else {
                    return Err(Error::ParsingError("unable to parse header".to_string()));
                }
            }
            else {
                break
            }
        }
        if value.len() < total+1 {
            return Err(Error::ParsingError("todo".to_string()));
        }
        req.raw_body = value[total+1..].as_bytes().to_vec();
        Ok(req)
    }

    pub fn proxy(mut self, loc: impl AsRef<str>) -> Result<Res> {
        let loc = loc.as_ref();
        todo!("Send request http/1.1 with is_proxy = true");
    }
    pub fn send(&self, loc: impl AsRef<str>) -> Result<Res> {
        let loc = loc.as_ref();
        if self.is_tls {
            let mut io = Tls::connect(loc).unwrap();
            let http1 = Http1::from(self);
        }
        Ok(Res::new()) 
    }

}
