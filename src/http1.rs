
pub struct Http1 {
    pub method: Vec<u8>,
    pub path: Vec<u8>,
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
    pub body_raw: Vec<u8>,
    pub separator: Vec<u8>,
    pub proxied: bool,
    pub host: Vec<u8>,
}

impl Http1 {
    pub fn new() -> Http1 {
        Http1 {
            method: b"GET".to_vec(),
            path: b"/".to_vec(),
            headers: vec![],
            body_raw: b"".to_vec(),
            separator: b"\r\n".to_vec(),
            host: b"".to_vec(),
            proxied: false,
        }
    }

    pub fn url(mut self, value: impl AsRef<[u8]>) -> Self {
        let value = value.as_ref();

        let mut l = value.len();
        for i in 0..value.len() {
            if value[i] == 47 {
                l = i;
                break;
            }
        }
        self.host = value[..l].to_vec();
        self.path = value[l..].to_vec();
        self.headers.insert(0, (b"Host".to_vec(), value[..l].to_vec()));
        self
    }

    pub fn header(mut self, header: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Self {
        self.headers.push((header.as_ref().to_vec(), value.as_ref().to_vec()));
        self
    }

    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref();
        self.body_raw = body.to_vec();
        self.header(b"Content-length", body.len().to_string())
    }

    pub fn raw(&self) -> Vec<u8> {
        let mut out = self.method.to_vec();
        out.extend(b" ");
        if self.proxied {
            out.extend(&self.host);
        }
        out.extend(&self.path);
        out.extend(b" HTTP/1.1");
        out.extend(&self.separator);
        for (header, value) in &self.headers {
            out.extend(header);
            out.extend(b": ");
            out.extend(value);
            out.extend(&self.separator);
        }
        out.extend(&self.separator);
        out.extend(&self.body_raw);
        out
    }

    pub fn proxy(mut self) -> Self {
        self.proxied = true;
        self
    }
}
