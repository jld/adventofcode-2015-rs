use std::hash::Hash;

trait Scanner: Hash + Eq {
    fn zero() -> Self;
    fn step(self, c: char) -> Self;
    fn nice(&self) -> bool;
}

fn nice<S: Scanner>(state: S, s: &str) -> bool {
    let mut state = state;
    for c in s.chars() {
        state = state.step(c);
    }
    state.nice()
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Vowels(u8);
const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];
const ENOUGH: u8 = 3;
impl Scanner for Vowels {
    fn zero() -> Vowels { Vowels(0) }
    fn step(self, c: char) -> Vowels {
        if self.0 >= ENOUGH {
            debug_assert_eq!(self.0, ENOUGH);
            self
        } else if VOWELS.iter().any(|&v| v == c) {
            Vowels(self.0 + 1)
        } else {
            self
        }
    }
    fn nice(&self) -> bool {
        debug_assert!(self.0 <= ENOUGH);
        self.0 >= ENOUGH
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Doubled {
    Nope,
    IfNext(u8),
    Yes,
}
impl Scanner for Doubled {
    fn zero() -> Doubled { Doubled::Nope }
    fn step(self, c: char) -> Doubled {
        match self {
            Doubled::Yes => Doubled::Yes,
            Doubled::IfNext(d) if c == (d as char) => Doubled::Yes,
            _ => {
                if c >= 'a' && c <= 'z' {
                    Doubled::IfNext(c as u8)
                } else {
                    Doubled::Nope
                }
            }
        }
    }
    fn nice(&self) -> bool {
        match *self {
            Doubled::Yes => true,
            _ => false
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Censor {
    Clean,
    Danger(u8),
    Naughty
}
const NONO: [[char; 2]; 4] = [['a', 'b'], ['c', 'd'], ['p', 'q'], ['x', 'y']];
impl Scanner for Censor {
    fn zero() -> Censor { Censor::Clean }
    fn step(self, c: char) -> Censor {
        match self {
            Censor::Naughty => Censor::Naughty,
            Censor::Danger(i) if c == NONO[i as usize][1]  => Censor::Naughty,
            _ => {
                for (i, nono) in NONO.iter().enumerate() {
                    if c == nono[0] {
                        return Censor::Danger(i as u8)
                    }
                }
                Censor::Clean
            }
        }
    }
    fn nice(&self) -> bool {
        match *self {
            Censor::Naughty => false,
            _ => true
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Both<S: Scanner, T: Scanner>(S, T);
impl<S: Scanner, T: Scanner> Scanner for Both<S, T> {
    fn zero() -> Both<S, T> {
        Both(S::zero(), T::zero())
    }
    fn step(self, c: char) -> Both<S, T> {
        Both(self.0.step(c), self.1.step(c))
    }
    fn nice(&self) -> bool {
        self.0.nice() && self.1.nice()
    }
}

type Santa = Both<Vowels, Both<Doubled, Censor>>;

#[cfg(test)]
mod test {
    use super::{Scanner, Vowels, Doubled, Censor, Santa, nice};

    #[test]
    fn spec_line1() {
        let word = "ugknbfddgicrmopn";
        assert!(nice(Vowels::zero(), word));
        assert!(nice(Doubled::zero(), word));
        assert!(nice(Censor::zero(), word));
        assert!(nice(Santa::zero(), word));
    }

    #[test]
    fn spec_line2() {
        let word = "aaa";
        assert!(nice(Vowels::zero(), word));
        assert!(nice(Doubled::zero(), word));
        assert!(nice(Censor::zero(), word));
        assert!(nice(Santa::zero(), word));
    }

    #[test]
    fn spec_line3() {
        let word = "jchzalrnumimnmhp";
        assert!(!nice(Doubled::zero(), word));
        assert!(!nice(Santa::zero(), word));
    }

    #[test]
    fn spec_line4() {
        let word = "haegwjzuvuyypxyu";
        assert!(!nice(Censor::zero(), word));
        assert!(!nice(Santa::zero(), word));
    }
    #[test]
    fn spec_line4b() {
        let word = "haegwjzuvuypxxyu";
        assert!(!nice(Censor::zero(), word));
        assert!(!nice(Santa::zero(), word));
    }

    #[test]
    fn spec_line5() {
        let word = "dvszwmarrgswjxmb";
        assert!(!nice(Vowels::zero(), word));
        assert!(!nice(Santa::zero(), word));
    }
}

