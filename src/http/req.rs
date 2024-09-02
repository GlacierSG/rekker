use crate::literal::bytes_to_lit_color;
use crate::{Error, Result, Pipe};
use crate::http::Res;
use colored::*;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub enum Version {
    H1_1,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Req {
    pub method: Vec<u8>,
    pub path: Vec<u8>,
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub body: Vec<u8>,
    pub url: Vec<u8>,
    pub version: Version,
    pub is_proxy: bool,
    pub tls: bool,
}

impl fmt::Display for Req {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn colored(b: &[u8]) -> String {
            bytes_to_lit_color(b, |x| x.into(), |x| x.yellow())
        }
        write!(f, "{}", colored(&self.method))?;
        write!(f, " ")?;
        if self.is_proxy {
            write!(f, "{}", colored(&self.url))?;
        }
        write!(f, "{}", colored(&self.path))?;
        match self.version {
            Version::H1_1 => write!(f, " HTTP/1.1\n")?,
            //Version::H2 => write!(f, " HTTP/2\n")?,
        }
        for (header, value) in &self.headers {
            write!(f, "{}", colored(&header))?;
            write!(f, ": ")?;
            write!(f, "{}", colored(&value))?;
            write!(f, "\n")?;
        }
        write!(f, "\n")?;
        write!(f, "{}", colored(&self.body))
    }
}

impl Req {
    pub fn new() -> Self {
        Self {
            method: vec![],
            path: vec![],
            url: vec![],
            headers: vec![],
            body: vec![],
            version: Version::H1_1,
            is_proxy: false,
            tls: false,
        }
    }

    pub fn raw(&self) -> Vec<u8> {
        let mut out = vec![];
        out.extend_from_slice(&self.method);
        out.push(10); // space
        if self.is_proxy {
            if self.tls {
                out.extend(b"https://");
            }
            else {
                out.extend(b"http://");
            }
            out.extend(&self.url);
        }
        out.extend_from_slice(&self.path);
        out.extend(b" HTTP/1.1");
        out.extend(b"\r\n");
        for (header, value) in &self.headers {
            out.extend(header);
            out.extend(b": ");
            out.extend(value);
            out.extend(b"\r\n");
        }
        out.extend(b"\r\n");
        out.extend_from_slice(&self.body);
        out
    }
    pub fn addr(&self) -> Result<String> {
        let mut addr = self.url.clone();
        if !addr.contains(&58) { // ':'
            addr.extend(b":80");
        }
        Ok(std::str::from_utf8(&addr).map_err(|_| Error::ParsingError("Could not convert addr to utf8".to_string()))?.to_string())
    }

    fn url(mut self, url: impl AsRef<[u8]>) -> Self {
        let url = url.as_ref();

        let mut t = 0;
        if url.len() >= 8 && &url[..8] == b"https://" {
            self.tls = true;
            t = 8;
        }
        else if url.len() >= 7 && &url[..7] == b"http://" {
            self.tls = false;
            t = 7;
        }

        let l = t + url.iter().skip(t).position(|&x| x == 47).unwrap_or(0); // Find next `/`
        self.url = url[t..l].to_vec();
        self.path = url[l..].to_vec();
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
        self.method = method.as_ref().to_vec();
        self
    }

    pub fn path(mut self, path: impl AsRef<[u8]>) -> Self {
        self.path = path.as_ref().to_vec();
        self
    }

    pub fn header(mut self, header: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Self {
        self.headers.push((header.as_ref().to_vec(), value.as_ref().to_vec()));
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref();
        self.body = body.to_vec();
        self
    }
    pub fn data(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref();
        self.body = body.to_vec();
        self.header(b"Content-Length", body.len().to_string())
    }

    pub fn proxy(mut self, addr: impl AsRef<str>) -> Result<Res> {
        let addr = addr.as_ref();
        let mut io = Pipe::tcp(&addr)?;
        let is_proxy = self.is_proxy;
        self.is_proxy = true;
        if let Err(e) = io.send(self.raw()) {
            self.is_proxy = is_proxy;
            return Err(e);
        }
        self.is_proxy = is_proxy;
        let res = Res::from_pipe(&mut io)?;
        Ok(res) 
    }
    pub fn send(&self) -> Result<Res> {
        let addr = self.addr()?;
        let mut io = Pipe::tcp(&addr)?;
        io.send(self.raw())?;
        let res = Res::from_pipe(&mut io)?;
        Ok(res) 
    }

}
