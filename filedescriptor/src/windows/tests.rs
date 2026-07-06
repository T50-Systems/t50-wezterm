#[cfg(test)]
mod test {
    use std::io::{Read, Write};

    #[test]
    fn socketpair() {
        let (mut a, mut b) = super::socketpair_impl().unwrap();
        a.write_all(b"hello").unwrap();
        let mut buf = [0u8; 5];
        assert_eq!(b.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"hello");
    }
}
