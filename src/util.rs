pub fn xor(lhs: impl AsRef<[u8]>, rhs: impl AsRef<[u8]>) -> Vec<u8> {
    let lhs = lhs.as_ref();
    let rhs = rhs.as_ref();
    let (lrg, sml) = if lhs.len() > rhs.len() {
        (lhs, rhs)
    } else {
        (rhs, lhs)
    };

    if sml.len() == 0 {
        return lrg.to_vec();
    }

    (0..lrg.len()).map(|i| lrg[i] ^ sml[i % sml.len()]).collect()
}

#[cfg(test)]
mod test {
    use super::xor;

    #[test]
    fn basic() {
        assert_eq!(xor(b"", b""), b"".to_vec());
        assert_eq!(xor(b"", b"abcdef"), b"abcdef".to_vec());
        assert_eq!(xor(b"abcdef", b""), b"abcdef".to_vec());
        assert_eq!(xor(b"abcdef", b"abcdef"), b"\x00\x00\x00\x00\x00\x00".to_vec());
    }
}
