use std::io::{stdin, Read};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Unquote,
    Quote,
    Backslash,
    Hex0,
    Hex1,
}

fn is_hexdigit(c: char) -> bool {
    match c {
        '0'...'9' | 'a'...'f' | 'A'...'F' => true,
        _ => false
    }
}

fn compute<I: Iterator<Item=char>>(chars: I) -> (u64, u64) {
    let mut waste = 0;
    let mut expand = 0;
    let mut state = State::Unquote;
    for char in chars {
        match (state, char) {
            (State::Unquote, '"') => { state = State::Quote; waste += 1; expand += 2 },
            (State::Unquote, _) => (),
            (State::Quote, '"') => { state = State::Unquote; waste += 1; expand += 2 },
            (State::Quote, '\\') => { state = State::Backslash; waste += 1; expand += 1 },
            (State::Quote, _) => (),
            (State::Backslash, '"') |
            (State::Backslash, '\\') => { state = State::Quote; expand += 1 },
            (State::Backslash, 'n') => { state = State::Quote; },
            (State::Backslash, 'x') => { state = State::Hex0; waste += 1 },
            (State::Backslash, _) => panic!("bad escape char {:?}", char),
            (State::Hex0, c) if is_hexdigit(c) => { state = State::Hex1; waste += 1 },
            (State::Hex1, c) if is_hexdigit(c) => { state = State::Quote;  },
            (State::Hex0, _) |
            (State::Hex1, _) => panic!("bad hexdigit {:?}", char),
        }
    }
    assert!(state == State::Unquote, "unterminated syntax (EOF in {:?})", state);
    (waste, expand)
}

pub fn main() {
    let stdin = stdin();
    // Sadly, io::Chars is unstable.
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf).unwrap();
    let (waste, expand) = compute(buf.chars());
    println!("Wasted {} char{}.", waste, if waste == 1 { "" } else { "s" });
    println!("Expanded by {} char{}.", expand, if expand == 1 { "" } else { "s" });
}

#[cfg(test)]
mod test {
    use super::compute;

    fn waste(s: &str) -> u64 { let (w, _e) = compute(s.chars()); w }
    fn expn(s: &str) -> u64 { let (_w, e) = compute(s.chars()); e }

    #[test]
    fn unquoted() {
        assert_eq!(waste("chicken"), 0);
        assert_eq!(waste("chicke\\n"), 0);
    }

    #[test]
    fn examples_severally() {
        assert_eq!(waste("\"\""), 2 - 0);
        assert_eq!(waste("\"abc\""), 5 - 3);
        assert_eq!(waste("\"aaa\\\"aaa\""), 10 - 7);
        assert_eq!(waste("\"\\x27\""), 6 - 1);
    }

    #[test]
    fn examples_jointly() {
        assert_eq!(waste("\"\"\n\"abc\"\n\"aaa\\\"aaa\"\n\"\\x27\"\n"), 23 - 11);
    }

    #[test]
    fn unquoted2() {
        assert_eq!(expn("chicken"), 0);
        assert_eq!(expn("chicke\\n"), 0);
    }

    #[test]
    fn examples2_severally() {
        assert_eq!(expn("\"\""), 6 - 2);
        assert_eq!(expn("\"abc\""), 9 - 5);
        assert_eq!(expn("\"aaa\\\"aaa\""), 16 - 10);
        assert_eq!(expn("\"\\x27\""), 11 - 6);
    }

    #[test]
    fn examples2_jointly() {
        assert_eq!(expn("\"\"\n\"abc\"\n\"aaa\\\"aaa\"\n\"\\x27\"\n"), 42 - 23);
    }
}
