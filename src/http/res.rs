use colored::*;
use crate::literal::bytes_to_lit_color;
use crate::{Pipe, Result, Error};
use std::fmt;

fn parse_usize(value: &[u8]) -> Result<usize> {
    let utf8 = std::str::from_utf8(value).map_err(|_| Error::ParsingError("Could not convert bytes to utf8".to_string()))?;
    Ok(utf8.parse::<usize>().map_err(|_| Error::ParsingError("Could not parse usize".to_string()))?)
}

#[derive(Clone, PartialEq, Eq)]
pub struct Res {
    pub version: Vec<u8>,
    pub code: Vec<u8>,
    pub reason: Vec<u8>,
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub body: Vec<u8>,
}


impl fmt::Display for Res {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn colored(b: &[u8]) -> String {
            bytes_to_lit_color(b, |x| x.into(), |x| x.yellow())
        }
        write!(f, "{}", colored(&self.version))?;
        write!(f, " ")?;
        write!(f, "{}", colored(&self.code))?;
        write!(f, " ")?;
        write!(f, "{}", colored(&self.reason))?;
        write!(f, "\n")?;
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

impl Res {
    pub fn new() -> Self {
        Self {
            code: vec![],
            reason: vec![],
            version: vec![],
            headers: vec![],
            body: vec![],
        }
    }
    pub fn status_code(&self) -> Result<u32> {
        let utf8 = std::str::from_utf8(&self.code).map_err(|_| Error::ParsingError("Could not convert status to utf8".to_string()))?;
        Ok(utf8.parse::<u32>().map_err(|_| Error::ParsingError("Could not parse status to u32".to_string()))?)
    }

    pub fn header(&mut self, header: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> () {
        self.headers.push((header.as_ref().to_vec(), value.as_ref().to_vec()));
    }

    pub fn from_pipe(io: &mut Pipe) -> Result<Res> {
        let mut res = Res::new();
        let sl = io.recvline()?;
        if sl.len() < 2 {
            return Err(Error::ParsingError("Missing status line".to_string()));
        }

        let sl = &sl[..sl.len()-1];
        let mut sl_split = sl[..sl.len()-1].split(|&x| x == 0x20);
        if let Some(version) = sl_split.next() {
            res.version = version.to_vec();
        }
        else {
            return Err(Error::ParsingError("Missing http version".to_string()));
        }

        if let Some(code) = sl_split.next() {
            res.code = code.to_vec();
        }
        else {
            return Err(Error::ParsingError("Missing status".to_string()));
        }


        let mut reason = vec![];
        for x in sl_split {
            reason.extend(x);
        }
        res.reason = reason;

        let headers = io.recvuntil(b"\r\n\r\n")?;
        let headers = &headers[..headers.len()-4];

        let mut content_len: usize = 0;

        let mut li = 0;
        loop {
            if let Some(ri) = headers[li..]
                            .windows(2)
                            .position(|w| w == b"\r\n") {
                if let Some(idx) = headers[li..li+ri].windows(2).position(|w| w == b": ") {
                    let header = &headers[li..li+idx];
                    let value = &headers[li+idx+2..li+ri];
                    res.header(header, value);
                    if header == b"Content-Length" {
                        content_len= parse_usize(value)?;
                    }
                }
                else {
                    return Err(Error::ParsingError("Could not parse header".to_string()));
                }
                li += ri + 2;
            }
            else {
                break;
            }
        }

        res.body = io.recvn(content_len)?;

        Ok(res)
    }
}


