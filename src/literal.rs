use colored::*;

pub fn to_lit_colored(bytes: impl AsRef<[u8]>, normal_fn: fn(&str) -> ColoredString, byte_fn: fn(&str) -> ColoredString) -> String {
    let bytes = bytes.as_ref();
    let mut lit = String::new();
    for byte in bytes {
        if *byte == 9  { lit.push_str(&format!("{}", byte_fn("\\t"))); }
        else if *byte == 10 { lit.push_str(&format!("{}", byte_fn("\\n"))); }
        else if *byte == 13 { lit.push_str(&format!("{}", byte_fn("\\r"))); }
        else if *byte == 92 { lit.push_str(&format!("{}", byte_fn("\\\\"))); }
        else if *byte >= 32 && *byte <= 126 {
            lit.push_str(&format!("{}", normal_fn(&String::from_utf8(vec![*byte]).unwrap())));
        }
        else {
            lit.push_str(&format!("{}", byte_fn(&format!("\\x{:02x}", byte))));
        }
    }
    lit
}

pub fn from_lit(input: impl AsRef<[u8]>) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let bytes = input.as_ref();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if i + 1 >= bytes.len() {
                return Err(format!("Incomplete escape sequence at {}", i));
            }
            match bytes[i + 1] {
                b'x' => {
                    if i + 4 > bytes.len() {
                        return Err(format!("Incomplete \\x escape sequence at {}", i));
                    }
                    let mut v = 0;
                    let mut m = 1;
                    for j in ((i + 2)..(i + 4)).rev() {
                        if 97 <= bytes[j] && bytes[j] <= 102 {
                            v += (bytes[j] - 97 + 10) * m;
                        } else if 48 <= bytes[j] && bytes[j] <= 57 {
                            v += (bytes[j] - 48) * m;
                        } else if 65 <= bytes[j] && bytes[j] <= 70 {
                            v += (bytes[j] - 65 + 10) * m;
                        } else {
                            return Err(format!("Incomplete \\x escape sequence at {}", i));
                        }
                        if j != i + 2 {
                            m *= 16;
                        }
                    }
                    result.push(v);
                    i += 4;
                }
                b'0' => {
                    result.push(b'\0');
                    i += 2;
                }
                b'n' => {
                    result.push(b'\n');
                    i += 2;
                }
                b'r' => {
                    result.push(b'\r');
                    i += 2;
                }
                b't' => {
                    result.push(b'\t');
                    i += 2;
                }
                _ => {
                    result.push(bytes[i + 1]);
                    i += 2;
                }
            }
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let raw = [
            r"\x00", r"\x01", r"\x02", r"\x03", r"\x04", r"\x05", r"\x06", r"\x07", r"\x08",
            r"\x09", r"\x0a", r"\x0b", r"\x0c", r"\x0d", r"\x0e", r"\x0f", r"\x10", r"\x11",
            r"\x12", r"\x13", r"\x14", r"\x15", r"\x16", r"\x17", r"\x18", r"\x19", r"\x1a",
            r"\x1b", r"\x1c", r"\x1d", r"\x1e", r"\x1f", r"\x20", r"\x21", r"\x22", r"\x23",
            r"\x24", r"\x25", r"\x26", r"\x27", r"\x28", r"\x29", r"\x2a", r"\x2b", r"\x2c",
            r"\x2d", r"\x2e", r"\x2f", r"\x30", r"\x31", r"\x32", r"\x33", r"\x34", r"\x35",
            r"\x36", r"\x37", r"\x38", r"\x39", r"\x3a", r"\x3b", r"\x3c", r"\x3d", r"\x3e",
            r"\x3f", r"\x40", r"\x41", r"\x42", r"\x43", r"\x44", r"\x45", r"\x46", r"\x47",
            r"\x48", r"\x49", r"\x4a", r"\x4b", r"\x4c", r"\x4d", r"\x4e", r"\x4f", r"\x50",
            r"\x51", r"\x52", r"\x53", r"\x54", r"\x55", r"\x56", r"\x57", r"\x58", r"\x59",
            r"\x5a", r"\x5b", r"\x5c", r"\x5d", r"\x5e", r"\x5f", r"\x60", r"\x61", r"\x62",
            r"\x63", r"\x64", r"\x65", r"\x66", r"\x67", r"\x68", r"\x69", r"\x6a", r"\x6b",
            r"\x6c", r"\x6d", r"\x6e", r"\x6f", r"\x70", r"\x71", r"\x72", r"\x73", r"\x74",
            r"\x75", r"\x76", r"\x77", r"\x78", r"\x79", r"\x7a", r"\x7b", r"\x7c", r"\x7d",
            r"\x7e", r"\x7f", r"\x80", r"\x81", r"\x82", r"\x83", r"\x84", r"\x85", r"\x86",
            r"\x87", r"\x88", r"\x89", r"\x8a", r"\x8b", r"\x8c", r"\x8d", r"\x8e", r"\x8f",
            r"\x90", r"\x91", r"\x92", r"\x93", r"\x94", r"\x95", r"\x96", r"\x97", r"\x98",
            r"\x99", r"\x9a", r"\x9b", r"\x9c", r"\x9d", r"\x9e", r"\x9f", r"\xa0", r"\xa1",
            r"\xa2", r"\xa3", r"\xa4", r"\xa5", r"\xa6", r"\xa7", r"\xa8", r"\xa9", r"\xaa",
            r"\xab", r"\xac", r"\xad", r"\xae", r"\xaf", r"\xb0", r"\xb1", r"\xb2", r"\xb3",
            r"\xb4", r"\xb5", r"\xb6", r"\xb7", r"\xb8", r"\xb9", r"\xba", r"\xbb", r"\xbc",
            r"\xbd", r"\xbe", r"\xbf", r"\xc0", r"\xc1", r"\xc2", r"\xc3", r"\xc4", r"\xc5",
            r"\xc6", r"\xc7", r"\xc8", r"\xc9", r"\xca", r"\xcb", r"\xcc", r"\xcd", r"\xce",
            r"\xcf", r"\xd0", r"\xd1", r"\xd2", r"\xd3", r"\xd4", r"\xd5", r"\xd6", r"\xd7",
            r"\xd8", r"\xd9", r"\xda", r"\xdb", r"\xdc", r"\xdd", r"\xde", r"\xdf", r"\xe0",
            r"\xe1", r"\xe2", r"\xe3", r"\xe4", r"\xe5", r"\xe6", r"\xe7", r"\xe8", r"\xe9",
            r"\xea", r"\xeb", r"\xec", r"\xed", r"\xee", r"\xef", r"\xf0", r"\xf1", r"\xf2",
            r"\xf3", r"\xf4", r"\xf5", r"\xf6", r"\xf7", r"\xf8", r"\xf9", r"\xfa", r"\xfb",
            r"\xfc", r"\xfd", r"\xfe", r"\xff",
        ];
        let bytes = [
            b"\x00", b"\x01", b"\x02", b"\x03", b"\x04", b"\x05", b"\x06", b"\x07", b"\x08",
            b"\x09", b"\x0a", b"\x0b", b"\x0c", b"\x0d", b"\x0e", b"\x0f", b"\x10", b"\x11",
            b"\x12", b"\x13", b"\x14", b"\x15", b"\x16", b"\x17", b"\x18", b"\x19", b"\x1a",
            b"\x1b", b"\x1c", b"\x1d", b"\x1e", b"\x1f", b"\x20", b"\x21", b"\x22", b"\x23",
            b"\x24", b"\x25", b"\x26", b"\x27", b"\x28", b"\x29", b"\x2a", b"\x2b", b"\x2c",
            b"\x2d", b"\x2e", b"\x2f", b"\x30", b"\x31", b"\x32", b"\x33", b"\x34", b"\x35",
            b"\x36", b"\x37", b"\x38", b"\x39", b"\x3a", b"\x3b", b"\x3c", b"\x3d", b"\x3e",
            b"\x3f", b"\x40", b"\x41", b"\x42", b"\x43", b"\x44", b"\x45", b"\x46", b"\x47",
            b"\x48", b"\x49", b"\x4a", b"\x4b", b"\x4c", b"\x4d", b"\x4e", b"\x4f", b"\x50",
            b"\x51", b"\x52", b"\x53", b"\x54", b"\x55", b"\x56", b"\x57", b"\x58", b"\x59",
            b"\x5a", b"\x5b", b"\x5c", b"\x5d", b"\x5e", b"\x5f", b"\x60", b"\x61", b"\x62",
            b"\x63", b"\x64", b"\x65", b"\x66", b"\x67", b"\x68", b"\x69", b"\x6a", b"\x6b",
            b"\x6c", b"\x6d", b"\x6e", b"\x6f", b"\x70", b"\x71", b"\x72", b"\x73", b"\x74",
            b"\x75", b"\x76", b"\x77", b"\x78", b"\x79", b"\x7a", b"\x7b", b"\x7c", b"\x7d",
            b"\x7e", b"\x7f", b"\x80", b"\x81", b"\x82", b"\x83", b"\x84", b"\x85", b"\x86",
            b"\x87", b"\x88", b"\x89", b"\x8a", b"\x8b", b"\x8c", b"\x8d", b"\x8e", b"\x8f",
            b"\x90", b"\x91", b"\x92", b"\x93", b"\x94", b"\x95", b"\x96", b"\x97", b"\x98",
            b"\x99", b"\x9a", b"\x9b", b"\x9c", b"\x9d", b"\x9e", b"\x9f", b"\xa0", b"\xa1",
            b"\xa2", b"\xa3", b"\xa4", b"\xa5", b"\xa6", b"\xa7", b"\xa8", b"\xa9", b"\xaa",
            b"\xab", b"\xac", b"\xad", b"\xae", b"\xaf", b"\xb0", b"\xb1", b"\xb2", b"\xb3",
            b"\xb4", b"\xb5", b"\xb6", b"\xb7", b"\xb8", b"\xb9", b"\xba", b"\xbb", b"\xbc",
            b"\xbd", b"\xbe", b"\xbf", b"\xc0", b"\xc1", b"\xc2", b"\xc3", b"\xc4", b"\xc5",
            b"\xc6", b"\xc7", b"\xc8", b"\xc9", b"\xca", b"\xcb", b"\xcc", b"\xcd", b"\xce",
            b"\xcf", b"\xd0", b"\xd1", b"\xd2", b"\xd3", b"\xd4", b"\xd5", b"\xd6", b"\xd7",
            b"\xd8", b"\xd9", b"\xda", b"\xdb", b"\xdc", b"\xdd", b"\xde", b"\xdf", b"\xe0",
            b"\xe1", b"\xe2", b"\xe3", b"\xe4", b"\xe5", b"\xe6", b"\xe7", b"\xe8", b"\xe9",
            b"\xea", b"\xeb", b"\xec", b"\xed", b"\xee", b"\xef", b"\xf0", b"\xf1", b"\xf2",
            b"\xf3", b"\xf4", b"\xf5", b"\xf6", b"\xf7", b"\xf8", b"\xf9", b"\xfa", b"\xfb",
            b"\xfc", b"\xfd", b"\xfe", b"\xff",
        ];
        for (r, b) in raw.iter().zip(bytes) {
            assert_eq!(&from_lit(r).unwrap(), b);
        }

        for (r, b) in raw.iter().zip(bytes) {
            let mut t1 = r.as_bytes().to_vec();
            t1.extend_from_slice(b"abc");
            let mut t2 = b.to_vec();
            t2.extend_from_slice(b"abc");
            assert_eq!(
                from_lit(t1).unwrap(),
                t2
            );
        }

        for (r, b) in raw.iter().zip(bytes) {
            let mut t1 = b"abc".to_vec();
            t1.extend_from_slice(r.as_bytes());
            let mut t2 = b"abc".to_vec();
            t2.extend_from_slice(b);
            assert_eq!(
                from_lit(t1).unwrap(),
                t2
            );
        }
    }
}

