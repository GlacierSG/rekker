//use std::collections::HashSet;
use super::{ssrf, xss, rce, sqli};
use crate::{Result, Error};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn generate_hex_string(n: usize, seed: u64) -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(seed);
    let random_bytes: Vec<u8> = (0..n).map(|_| rng.gen::<u8>()).collect();
    let s: String = random_bytes.iter().map(|byte| format!("{:02x}", byte)).collect();
    s.as_bytes().to_vec()
}



/*#[derive(Eq, Hash, PartialEq)]
pub enum InjType {
    Curl,
    Ssrf,
    Rce,
    //Sleep,
    //Sql,
}*/

pub struct Injector {
    pub file: Option<Vec<u8>>,
    pub host: Option<Vec<u8>>,
    pub port: Option<u16>,
    pub use_http: bool,
    pub use_https: bool,
    //pub tags: HashSet<InjType>,
}

impl Injector {
    pub fn new() -> Injector {
        Injector {
            file: None,
            host: None,
            port: None,
            use_http: true,
            use_https: true,
        }
    }

    pub fn canary(&self, n: u64) -> Vec<u8> {
        generate_hex_string(3, n)
    }

    pub fn port(&self) -> Option<Vec<u8>> {
        if let Some(port) = &self.port {
            Some(port.to_string().as_bytes().to_vec())
        }
        else {
            None
        }
    }
    pub fn addr(&self) -> Option<Vec<u8>> {
        if let Some(host) = &self.host {
            let mut out = host.clone();
            if let Some(port) = self.port() {
                out.extend(b":");
                out.extend(port);
            }
            return Some(out);
        }
        else {
            return None;
        }
    }
}

impl Injector {
    pub fn payload(&self, n: u64) -> Result<Vec<u8>> {
        let mut i = 0;
        match sqli::inj(&self, i, n) { 
            Ok(out) => return Ok(out),
            Err(ith) => i = ith
        }
        match ssrf::inj(&self, i, n) { 
            Ok(out) => return Ok(out),
            Err(ith) => i = ith
        }
        match rce::inj(&self, i, n) { 
            Ok(out) => return Ok(out),
            Err(ith) => i = ith
        }
        match xss::inj(&self, i, n) { 
            Ok(out) => return Ok(out),
            Err(ith) => i = ith
        }
        Err(Error::Invalid("n is too large".to_string()))
    }
}
