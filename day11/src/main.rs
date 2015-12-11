use std::io::{stdin,BufRead};

fn is_okay(b: u8) -> bool {
    match b as char {
        'i' | 'l' | 'o' => false,
        'a'...'z' => true,
        c => panic!("Not a letter: {:?}", c)
    }
}

fn zap(sl: &mut[u8]) {
    let a = 'a' as u8;
    assert!(is_okay(a));
    for p in sl {
        *p = a;
    }
}

fn incr(sl: &mut[u8]) {
    assert!(sl.len() > 0, "Can't increment empty slice");
    let last_idx = sl.len() - 1;
    let (butlast, last) = sl.split_at_mut(last_idx);
    loop {
        if last[0] as char == 'z' {
            zap(last);
            incr(butlast);
            break;
        }
        last[0] += 1;
        if is_okay(last[0]) {
            break;
        }
    }
}

fn incr_to_okay(sl: &mut[u8]) {
    for i in 0..sl.len() {
        if !is_okay(sl[i]) {
            let (to_incr,to_zap) = sl.split_at_mut(i+1);
            incr(to_incr);
            zap(to_zap);
            return;
        }
    }
    incr(sl);
}

fn has_run(sl: &[u8], rl: usize) -> bool {
    (rl-1..sl.len()).any(|i| {
        let i = i - (rl - 1);
        (1..rl).all(|j| {
            sl[i + j] as usize == sl[i] as usize + j
        })
    })
}

fn has_tuples(sl: &[u8], rl: usize, nt: usize) -> bool {
    // Excitingly unspecified case: does "aaa" contain two pair of
    // letters?  This implementation assumes not.
    nt == 0 || (rl-1..sl.len()).any(|i| {
        let i = i - (rl - 1);
        (1..rl).all(|j| {
            sl[i + j] == sl[i]
        }) && (has_tuples(&sl[i+rl..], rl, nt - 1) || return false)
        // Yes, that typechecks.  No, it doesn't change the functional
        // behavior.  Yes, it's there for a reason: because that way
        // the runtime is linear in (`sl.len() + rl*nt`) instead of
        // exponential in `nt` (but you can kind of ignore it and
        // pretend everything is still pretty and functional).
    })
}

fn nextpass_mut(pw: &mut [u8]) {
    incr_to_okay(pw);
    while !(has_run(pw, 3) &&
            has_tuples(pw, 2, 2)) {
        incr(pw);
    }
}

fn apply_mut<F: FnOnce(&mut [u8])>(f: F, s: &str) -> String {
    let mut buf = s.to_owned().into_bytes();
    f(&mut buf);
    String::from_utf8(buf).unwrap()
}

pub fn nextpass(s: &str) -> String {
    apply_mut(nextpass_mut, s)
}

pub fn main() {
    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("I/O error reading stdin");
        println!("{}", nextpass(&line));
    }
}

#[cfg(test)]
mod test {
    use super::{apply_mut, is_okay, incr, incr_to_okay, has_run, has_tuples, nextpass};

    fn rule1(s: &str) -> bool { has_run(s.as_bytes(), 3) }
    fn rule2(s: &str) -> bool { s.as_bytes().iter().all(|&b| is_okay(b)) }
    fn rule3(s: &str) -> bool { has_tuples(s.as_bytes(), 2, 2) }

    #[test]
    fn test_incr() {
        const EXAMPLE: &'static[&'static str] = &[ "xx", "xy", "xz", "ya", "yb" ];
        for i in 1..EXAMPLE.len() {
            assert_eq!(apply_mut(incr, EXAMPLE[i-1]), EXAMPLE[i]);
        }
    }

    #[test]
    fn test_incr_to_okay() {
        assert_eq!(apply_mut(incr_to_okay, "abcdffaa"), "abcdffab");
        assert_eq!(apply_mut(incr_to_okay, "abcdffai"), "abcdffaj");
        assert_eq!(apply_mut(incr_to_okay, "ghijklmn"), "ghjaaaaa");
    }

    #[test]
    fn rule1_examples() {
        assert!(rule1("abc"));
        assert!(rule1("bcd"));
        assert!(rule1("cde"));
        assert!(rule1("xyz"));
        assert!(!rule1("abd"));
        assert!(rule1("hijklmmn"));
        assert!(!rule1("abbceffg"));
        assert!(rule1("abcdffaa"));
        assert!(rule1("ghjaabcc"));
    }

    #[test]
    fn rule2_examples() {
        assert!(!rule2("hijklmmn"));
        assert!(rule2("abcdffaa"));
        assert!(!rule2("ghijklmn"));
        assert!(rule2("ghjaabcc"));
    }

    #[test]
    fn rule2_hax() {
        assert!(!rule2("ghi....."));
        assert_eq!(apply_mut(incr_to_okay, "ghi....."), "ghjaaaaa");
    }
    #[test] #[should_panic(expected = "Not a letter: '.'")]
    fn unletter_rule2() {
        assert!(rule2("ghj....."));
    }
    #[test] #[should_panic(expected = "Not a letter: '.'")]
    fn unletter_i2o() {
        assert_eq!(apply_mut(incr_to_okay, "ghj....."), "ghj..../");
    }

    #[test]
    fn rule3_example() {
        assert!(rule3("abbceffg"));
        assert!(!rule3("abbcegjk"));
        assert!(rule3("abcdffaa"));
        assert!(rule3("ghjaabcc"));
        assert!(!rule3("aaa")); // Warning: kind of unspecified; see above.
    }

    #[test]
    fn nextpass_example() {
        assert_eq!(nextpass("abcdefgh"), "abcdffaa");
        assert_eq!(nextpass("ghijklmn"), "ghjaabcc");
    }
}
