use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::marker::PhantomData;

trait Scanner {
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


#[derive(Clone, Debug)]
struct Tabulate<S: Scanner + Hash + Eq + Clone> {
    tab: Rc<Tables>,
    state: Idx,
    // So this is kind of sketchy:
    phantom: PhantomData<S>
}
#[derive(PartialEq, Eq, Debug)]
struct Tables {
    step: Box<[[Idx; LETTERS]]>,
    nice: Box<[bool]>,
}
const LETTERS: usize = 27;
const LBASE: char = '`';
type Idx = u16;
impl Tables {
    fn lidx_char(i: usize) -> char { (LBASE as u8 + i as u8) as char }
    fn char_lidx(c: char) -> usize {
        let i = (c as usize).wrapping_sub(LBASE as usize);
        if i < LETTERS { i } else { 0 }
    }
    fn make<S: Scanner + Hash + Eq + Clone>() -> Tables {
        let z = S::zero();
        let mut stoi = HashMap::new();
        let mut itos = Vec::new();
        let mut i = 0;
        let mut step_acc = Vec::new();
        let mut nice_acc = Vec::new();
        stoi.insert(z.clone(), 0);
        itos.push(z);
        while i < itos.len() {
            let s = itos[i].clone();
            let mut step = [!0; LETTERS]; // Sigh.
            for l in 0..LETTERS {
                let sl = s.clone().step(Tables::lidx_char(l));
                step[l] = *(stoi.entry(sl.clone()).or_insert_with(|| {
                    let il = itos.len() as Idx;
                    itos.push(sl);
                    il
                }));
            }
            i += 1;
            step_acc.push(step);
            nice_acc.push(s.nice());
        }
        assert_eq!(i, step_acc.len());
        assert_eq!(i, nice_acc.len());
        Tables {
            step: step_acc.into_boxed_slice(),
            nice: nice_acc.into_boxed_slice(),
        }
    }
}

impl<S: Scanner + Hash + Eq + Clone> Scanner for Tabulate<S> {
    fn zero() -> Tabulate<S> {
        Tabulate {
            tab: Rc::new(Tables::make::<S>()),
            state: 0,
            phantom: PhantomData
        }
    }
    fn step(self, c: char) -> Tabulate<S> {
        let next = self.tab.step[self.state as usize][Tables::char_lidx(c)];
        Tabulate { state: next, ..self }
    }
    fn nice(&self) -> bool {
        self.tab.nice[self.state as usize]
    }
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
type FastSanta = Tabulate<Santa>;

#[cfg(test)]
mod test {
    use super::{Scanner, Vowels, Doubled, Censor, Santa, FastSanta, nice};

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

    #[test]
    fn fast_santa() {
        let z = FastSanta::zero();
        assert!(nice(z.clone(), "ugknbfddgicrmopn"));
        assert!(nice(z.clone(), "aaa"));
        assert!(!nice(z.clone(), "jchzalrnumimnmhp"));
        assert!(!nice(z.clone(), "haegwjzuvuyypxyu"));
        assert!(!nice(z.clone(), "haegwjzuvuypxxyu"));
        assert!(!nice(z.clone(), "dvszwmarrgswjxmb"));
    }
}

