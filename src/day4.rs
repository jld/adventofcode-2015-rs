extern crate md5;
// use std::io::{stdin, BufRead};

fn fmt_gen(b: &mut[u8], n: u64, radix: u64, zero: u8) -> &[u8] {
    let mut i = b.len();
    let mut a = n;
    while a != 0 {
        assert!(i > 0, "number {} too long for buffer size {}", n, b.len());
        i -= 1;
        b[i] = zero + (a % radix) as u8;
        a /= radix;
    }
    &b[i..]
}
fn fmt_num(b: &mut[u8], n: u64) -> &[u8] {
    fmt_gen(b, n, 10, '0' as u8)
}
fn fmt_lc(b: &mut[u8], n: u64) -> &[u8] {
    #![allow(dead_code)]
    fmt_gen(b, n, 26, 'a' as u8)
}

fn md5_is_zpfx(d: md5::Digest, nz: usize) -> bool {
    for i in 0..nz/2 {
        if d[i] != 0 {
            return false;
        }
    }
    if nz % 2 == 1 && d[nz / 2] >> 4 != 0  {
        return false;
    }
    return true;
}

/*
fn compute(s: &str) {
    let mut ctx = md5::Context::new();
    ctx.consume(s.as_bytes());
}
 */

#[cfg(test)]
mod test {
    extern crate md5;
    use super::{fmt_num,fmt_lc,md5_is_zpfx};
    
    #[test]
    fn test_fmt_num() {
        let mut buf = [0u8; 20];
        assert_eq!(fmt_num(&mut buf, 1048576), "1048576".as_bytes());
        assert_eq!(fmt_num(&mut buf, 999), "999".as_bytes());
        assert_eq!(fmt_num(&mut buf, 0), "".as_bytes());
        assert_eq!(fmt_num(&mut buf, 10000000000000000000), "10000000000000000000".as_bytes());
        assert_eq!(fmt_num(&mut buf, 18446744073709551615), "18446744073709551615".as_bytes());
    }

    #[test]
    #[should_panic(expected = "number 10000000000000000000 too long for buffer size 19")]
    fn test_fmt_tooshort() {
        let mut buf = [0u8; 19];
        assert_eq!(fmt_num(&mut buf, 9999999999999999999), "9999999999999999999".as_bytes());
        assert_eq!(fmt_num(&mut buf, 10000000000000000000), "10000000000000000000".as_bytes());
    }

    #[test]
    fn test_fmt_lc() {
        let mut buf = [0u8; 13];
        assert_eq!(fmt_lc(&mut buf, 0), "".as_bytes());
        assert_eq!(fmt_lc(&mut buf, 1), "b".as_bytes());
        assert_eq!(fmt_lc(&mut buf, 26), "ba".as_bytes());
        assert_eq!(fmt_lc(&mut buf, 1351), "bzz".as_bytes());
        assert_eq!(fmt_lc(&mut buf, 1067690712611132653), "lexicographer".as_bytes());
    }

    fn zpfx(s: &str, n: usize) -> bool {
        md5_is_zpfx(md5::compute(s.as_bytes()), n)
    }
    
    #[test]
    fn test_zpfx_spec() {
        assert!(zpfx("abcdef609043", 5));
        assert!(zpfx("pqrstuv1048970", 5));
    }
    #[test]
    fn test_zpfx_shorter() {
        for i in 0..5 {
            assert!(zpfx("abcdef609043", i));
            assert!(zpfx("pqrstuv1048970", i));
        }
    }
    #[test]
    fn test_zpfx_not() {
        for i in 6..9 {
            assert!(!zpfx("abcdef609043", i));
            assert!(!zpfx("pqrstuv1048970", i));
        }
    }
    #[test]
    fn test_zpfx_even() {
        assert!(zpfx("abcdef298", 2));
        assert!(!zpfx("abcdef298", 3));
    }
}
