use std::collections::HashMap;
use std::env;
use std::io::{stdin, BufRead};
use std::hash::Hash;
use std::rc::Rc;

trait Scanner {
    fn step(self, c: char) -> Self;
    fn nice(&self) -> bool;
}
trait ZScanner: Scanner {
    fn zero() -> Self;
}

fn nice<S: Scanner>(state: S, s: &str) -> bool {
    let mut state = state;
    for c in s.chars() {
        state = state.step(c);
    }
    state.nice()
}


#[derive(Clone, Debug)]
struct Tabulate {
    tab: Rc<Tables>,
    state: Idx,
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
    fn build<S: Scanner + Hash + Eq + Clone>(z: S) -> Tables {
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

impl Tabulate {
    fn new<S: Scanner + Hash + Eq + Clone>(s: S) -> Tabulate {
        Tabulate {
            tab: Rc::new(Tables::build(s)),
            state: 0,
        }
    }
}
impl Scanner for Tabulate {
    fn step(self, c: char) -> Tabulate {
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
impl ZScanner for Vowels {
    fn zero() -> Vowels { Vowels(0) }
}
impl Scanner for Vowels {
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
impl ZScanner for Doubled {
    fn zero() -> Doubled { Doubled::Nope }
}
impl Scanner for Doubled {
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
enum Censor<S: Scanner> {
    Clean(S),
    Danger(S, u8),
    Naughty
}
const NONO: [[char; 2]; 4] = [['a', 'b'], ['c', 'd'], ['p', 'q'], ['x', 'y']];
impl<S: ZScanner> ZScanner for Censor<S> {
    fn zero() -> Censor<S> { Censor::Clean(S::zero()) }
}
impl<S: Scanner> Scanner for Censor<S> {
    fn step(self, c: char) -> Censor<S> {
        match self {
            Censor::Naughty => Censor::Naughty,
            Censor::Danger(_, i) if c == NONO[i as usize][1]  => Censor::Naughty,
            Censor::Danger(s, _) | Censor::Clean(s) => {
                let s = s.step(c);
                for (i, nono) in NONO.iter().enumerate() {
                    if c == nono[0] {
                        return Censor::Danger(s, i as u8)
                    }
                }
                Censor::Clean(s)
            }
        }
    }
    fn nice(&self) -> bool {
        match *self {
            Censor::Naughty => false,
            Censor::Danger(ref s, _) | Censor::Clean(ref s) => s.nice()
        }
    }
}

#[derive(Clone, Debug)]
enum DoubleTrouble {
    Looking { pairmap: BigramSet, last: (u8, u8) },
    Found
}
impl ZScanner for DoubleTrouble {
    fn zero() -> DoubleTrouble {
        DoubleTrouble::Looking {
            pairmap: BigramSet::empty(),
            last: (0, 0)
        }
    }
}
#[derive(Clone, Debug)]
struct BigramSet { bits: [u32; 26] }
impl BigramSet {
    fn empty() -> BigramSet { BigramSet{ bits: [0; 26] }}
    fn contains(&self, l0: u8, l1: u8) -> bool {
        l0 != 0 && l1 != 0 && self.bits[l0 as usize - 1] & 1 << l1 - 1 != 0
    }
    fn add(&mut self, l0: u8, l1: u8) {
        if l0 != 0 && l1 != 0 {
            self.bits[l0 as usize - 1] |= 1 << l1 - 1
        }
    }
}
impl Scanner for DoubleTrouble {
    fn step(self, c: char) -> DoubleTrouble {
        match self {
            DoubleTrouble::Found => DoubleTrouble::Found,
            DoubleTrouble::Looking { mut pairmap, last: (l0, l1) } => {
                let l2 = Tables::char_lidx(c) as u8;
                if pairmap.contains(l1, l2) {
                    DoubleTrouble::Found
                } else {
                    // If this were before the test, then "aaa" would be nice.
                    pairmap.add(l0, l1);
                    DoubleTrouble::Looking { pairmap: pairmap, last: (l1, l2) }
                }
            }
        }
    }
    fn nice(&self) -> bool {
        match *self {
            DoubleTrouble::Found => true,
            _ => false
        }
    }
}

// What's round on both sides and high in the middle?  A camel.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Camel {
    Starting,
    Accelerating(char),
    Foraging(char, char),
    Resting,
}
// ...yes, I know that's how the joke is *supposed* to go, I mean I
// lived in Ohio for four years and all, but... oh, never mind.
impl ZScanner for Camel {
    fn zero() -> Camel {
        Camel::Starting
    }
}
impl Scanner for Camel {
    fn step(self, c: char) -> Camel {
        match self {
            Camel::Starting => Camel::Accelerating(c),
            Camel::Accelerating(b) => Camel::Foraging(b, c),
            Camel::Foraging(a, b) if a != c => Camel::Foraging(b, c),
            _ => Camel::Resting
        }
    }
    fn nice(&self) -> bool {
        match *self {
            Camel::Resting => true,
            _ => false
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Both<S: Scanner, T: Scanner>(S, T);
impl<S: ZScanner, T: ZScanner> ZScanner for Both<S, T> {
    fn zero() -> Both<S, T> {
        Both(S::zero(), T::zero())
    }
}
impl<S: Scanner, T: Scanner> Scanner for Both<S, T> {
    fn step(self, c: char) -> Both<S, T> {
        Both(self.0.step(c), self.1.step(c))
    }
    fn nice(&self) -> bool {
        self.0.nice() && self.1.nice()
    }
}

type Santa = Censor<Both<Vowels, Doubled>>;
fn slow_santa() -> Santa { Santa::zero() }
fn fast_santa() -> Tabulate { Tabulate::new(slow_santa()) }
type SantaTwo = Both<DoubleTrouble, Camel>;
fn new_santa() -> SantaTwo { SantaTwo::zero() }

fn dump() {
    let z = fast_santa();
    for i in 0..z.tab.step.len() {
        let nn = if z.tab.nice[i] { "nice" } else { "naughty" };
        let mut tbuf = "[".to_owned();
        for j in 0..LETTERS {
            tbuf.push_str(&format!("{}, ", z.tab.step[i][j]));
        }
        tbuf.push(']');
        println!("{} {} => {}", i, nn, tbuf);
    }
}
fn checker<S: Scanner + Clone>(s: S) {
    let stdin = stdin();
    let mut count: u64 = 0;
    for line in stdin.lock().lines() {
        let line = line.expect("I/O error reading stdin");
        if nice(s.clone(), &line) {
            count += 1;
        }
    }
    println!("{} string{} nice.", count, if count == 1 { " is" } else { "s are" });
}

pub fn main() {
    let argv1 = env::args().nth(1);
    match argv1.as_ref().map(|s| s as &str /* Sigh. */).unwrap_or("fast") {
        "dump" => dump(),
        "slow" => checker(slow_santa()),
        "fast" => checker(fast_santa()),
        huh => panic!("unknown command {}", huh)
    }
}

#[cfg(test)]
mod test {
    use super::{Scanner, ZScanner, Vowels, Doubled, Censor, Santa, Both, Tabulate,
                DoubleTrouble, Camel, nice, fast_santa, new_santa};

    fn naughty<S: Scanner>(state: S, s: &str) -> bool { !nice(state, s) }

    struct Oprah;
    impl ZScanner for Oprah {
        fn zero() -> Oprah { Oprah }
    }
    impl Scanner for Oprah {
        fn step(self, _: char) -> Oprah { self }
        fn nice(&self) -> bool { true }
    }
    type CensorTest = Censor<Oprah>;

    #[test]
    fn spec_line1() {
        let word = "ugknbfddgicrmopn";
        assert!(nice(Vowels::zero(), word));
        assert!(nice(Doubled::zero(), word));
        assert!(nice(CensorTest::zero(), word));
        assert!(nice(Santa::zero(), word));
    }

    #[test]
    fn spec_line2() {
        let word = "aaa";
        assert!(nice(Vowels::zero(), word));
        assert!(nice(Doubled::zero(), word));
        assert!(nice(CensorTest::zero(), word));
        assert!(nice(Santa::zero(), word));
    }

    #[test]
    fn spec_line3() {
        let word = "jchzalrnumimnmhp";
        assert!(naughty(Doubled::zero(), word));
        assert!(naughty(Santa::zero(), word));
    }

    #[test]
    fn spec_line4() {
        let word = "haegwjzuvuyypxyu";
        assert!(naughty(CensorTest::zero(), word));
        assert!(naughty(Santa::zero(), word));
    }
    #[test]
    fn spec_line4b() {
        let word = "haegwjzuvuypxxyu";
        assert!(naughty(CensorTest::zero(), word));
        assert!(naughty(Santa::zero(), word));
    }

    #[test]
    fn spec_line5() {
        let word = "dvszwmarrgswjxmb";
        assert!(naughty(Vowels::zero(), word));
        assert!(naughty(Santa::zero(), word));
    }

    #[test]
    fn fast_specs() {
        let z = fast_santa();
        assert!(nice(z.clone(), "ugknbfddgicrmopn"));
        assert!(nice(z.clone(), "aaa"));
        assert!(naughty(z.clone(), "jchzalrnumimnmhp"));
        assert!(naughty(z.clone(), "haegwjzuvuyypxyu"));
        assert!(naughty(z.clone(), "haegwjzuvuypxxyu"));
        assert!(naughty(z.clone(), "dvszwmarrgswjxmb"));
    }

    #[test]
    fn tab_functional() {
        assert_eq!(fast_santa().tab, fast_santa().tab);
    }

    #[test]
    fn tab_extensional() {
        type T0 = Both<Vowels, Doubled>;
        type T1 = Both<Doubled, Vowels>;
        let t0 = Tabulate::new(T0::zero());
        let t1 = Tabulate::new(T1::zero());
        assert_eq!(t0.tab, t1.tab);
    }

    #[test]
    fn dt_spec() {
        let z = DoubleTrouble::zero();
        assert!(nice(z.clone(), "xyxy"));
        assert!(nice(z.clone(), "aabcdefgaa"));
        assert!(naughty(z.clone(), "aaa"));
        assert!(nice(z.clone(), "qjhvhtzxzqqjkmpb"));
        assert!(nice(z.clone(), "xxyxx"));
        assert!(nice(z.clone(), "uurcxstgmygtbstg"));
        assert!(naughty(z.clone(), "ieodomkazucvgmuy"));
    }

    #[test]
    fn cm_spec() {
        let z = Camel::zero();
        assert!(nice(z.clone(), "xyx"));
        assert!(nice(z.clone(), "abcdefeghi"));
        assert!(nice(z.clone(), "aaa"));
        assert!(nice(z.clone(), "qjhvhtzxzqqjkmpb"));
        assert!(nice(z.clone(), "xxyxx"));
        assert!(naughty(z.clone(), "uurcxstgmygtbstg"));
        assert!(nice(z.clone(), "ieodomkazucvgmuy"));
    }

    #[test]
    fn s2_spec() {
        let z = new_santa();
        assert!(nice(z.clone(), "qjhvhtzxzqqjkmpb"));
        assert!(nice(z.clone(), "xxyxx"));
        assert!(naughty(z.clone(), "uurcxstgmygtbstg"));
        assert!(naughty(z.clone(), "ieodomkazucvgmuy"));
    }
}

