use crate::{Req, Res};

pub struct Http1 {
    data: Vec<u8>
}

impl From<&Req> for Http1 {
    fn from(req: &Req) -> Self {
        let mut out = vec![];
        out.extend(&req.raw_method);
        if req.is_proxy {
            out.extend(&req.raw_url);
        }
        out.extend(&req.raw_path);
        out.extend(b" HTTP/1.1\n");
        for (header, value) in &req.raw_headers {
            out.extend(header);
            out.extend(b": ");
            out.extend(value);
            out.extend(b"\n");
        }
        out.extend(b"\n");
        out.extend(&req.raw_body);
        Http1{ data: out }
    }
}

impl Http1 {
    pub fn send(req: &Req) {
        let http1 = Http1::from(req);
    }
}
