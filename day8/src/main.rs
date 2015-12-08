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

fn compute<I: Iterator<Item=char>>(chars: I) -> u64 {
    let mut waste = 0;
    let mut state = State::Unquote;
    for char in chars {
        let wasted = match (state, char) {
            (State::Unquote, '"') => { state = State::Quote; true },
            (State::Unquote, _) => false,
            (State::Quote, '"') => { state = State::Unquote; true },
            (State::Quote, '\\') => { state = State::Backslash; true },
            (State::Quote, _) => false,
            (State::Backslash, '\\') |
            (State::Backslash, 'n') |
            (State::Backslash, '"') => { state = State::Quote; false },
            (State::Backslash, 'x') => { state = State::Hex0; true },
            (State::Backslash, _) => panic!("bad escape char {:?}", char),
            (State::Hex0, c) if is_hexdigit(c) => { state = State::Hex1; true },
            (State::Hex1, c) if is_hexdigit(c) => { state = State::Quote; false },
            (State::Hex0, _) |
            (State::Hex1, _) => panic!("bad hexdigit {:?}", char),
        };
        if wasted {
            waste += 1;
        }
    }
    assert!(state == State::Unquote, "unterminated syntax (EOF in {:?})", state);
    waste
}

pub fn main() {
    let stdin = stdin();
    // Sadly, io::Chars is unstable.
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf).unwrap();
    let waste = compute(buf.chars());
    println!("Wasted {} char{}.", waste, if waste == 1 { "" } else { "s" });
}

#[cfg(test)]
mod test {
    use super::compute;

    #[test]
    fn unquoted() {
        assert_eq!(compute("chicken".chars()), 0);
        assert_eq!(compute("chicke\\n".chars()), 0);
    }

    #[test]
    fn examples_severally() {
        assert_eq!(compute("\"\"".chars()), 2 - 0);
        assert_eq!(compute("\"abc\"".chars()), 5 - 3);
        assert_eq!(compute("\"aaa\\\"aaa\"".chars()), 10 - 7);
        assert_eq!(compute("\"\\x27\"".chars()), 6 - 1);
    }
    
    #[test]
    fn examples_jointly() {
        assert_eq!(compute("\"\"\n\"abc\"\n\"aaa\\\"aaa\"\n\"\\x27\"\n".chars()), 23 - 11);
    }
}
