use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::str::FromStr;

#[derive(Debug)]
pub struct SymTab {
    rname: HashMap<String, usize>,
    pname: Vec<String>,
}
impl SymTab {
    pub fn new() -> SymTab { SymTab {
        rname: HashMap::new(),
        pname: Vec::new(),
    }}
    pub fn read(&mut self, s: &str) -> usize {
        // This clones the string unnecessarily if it's already
        // present.  Using `contains_key` and `insert` unecessarily
        // hashes the string twice if it's not.  Such is life.
        let entry = self.rname.entry(s.to_owned());
        let pname = &mut self.pname;
        *entry.or_insert_with(|| {
            let n = pname.len();
            pname.push(s.to_owned());
            n
        })
    }
    pub fn print(&self, n: usize) -> String {
        self.pname[n].clone()
    }
}

pub type Grid<N> = Box<[Box<[Option<N>]>]>;

trait Autovivify<K, V: Default> {
    fn autovivify(&mut self, k: K) -> &mut V;
}

impl<T: Default> Autovivify<usize, T> for Vec<T> {
    fn autovivify(&mut self, n: usize) -> &mut T {
        while n >= self.len() {
            self.push(T::default());
        }
        &mut self[n]
    }
}

fn emplace_or_else<T, F: FnOnce(&mut T, T)>(o: &mut Option<T>, v: T, f: F) {
    if let Some(p) = o.as_mut() {
        return f(p, v);
    }
    *o = Some(v);
}

pub fn parse<N, B>(stab: &mut SymTab, b: B) -> Grid<N>
    where N: FromStr + Copy, N::Err: Debug, B: BufRead {
    let mut acc: Vec<Vec<Option<N>>> = Vec::new();
    for (ln, line) in b.lines().enumerate() {
        let line = line.expect("I/O error");
        let words: Vec<_> = line.split(char::is_whitespace).filter(|s| !s.is_empty()).collect();
        assert!(words.len() >= 5, "line {}: too few tokens", ln);
        assert!(words.len() <= 5, "line {}: too many tokens", ln);
        assert!(words[1] == "to", "line {}: expected \"to\", got {:?}", ln, words[1]);
        assert!(words[3] == "=", "line {}: expected \"=\", got {:?}", ln, words[3]);
        let nsrc = stab.read(words[0]);
        let ndst = stab.read(words[2]);
        assert!(nsrc != ndst, "line {}: bad distance from {:?} to itself", ln, words[0]);
        debug_assert!(words[0] != words[2]);
        let dist = N::from_str(words[4]).unwrap_or_else(|e| {
            panic!("line {}: {:?} is not a number: {:?}", ln, words[4], e);
        });
        emplace_or_else(acc.autovivify(nsrc).autovivify(ndst), dist, |_p, _v| {
            panic!("line {}: duplicate distance from {} to {}", ln, words[0], words[2]);
        });
        emplace_or_else(acc.autovivify(ndst).autovivify(nsrc), dist, |_p, _v| {
            unreachable!();
        });
    }
    let rows: Vec<_> = acc.into_iter().map(Vec::into_boxed_slice).collect();
    rows.into_boxed_slice()
}

#[cfg(test)]
mod test {
    use super::{parse, Grid, SymTab};

    #[test]
    fn repeated_read() {
        let mut st = SymTab::new();
        let c0 = st.read("Coals");
        let n0 = st.read("Newcastle");
        let n1 = st.read("Newcastle");
        let c1 = st.read("Coals");
        assert_eq!(c0, c1);
        assert_eq!(n0, n1);
    }

    #[test]
    fn read_print() {
        let mut st = SymTab::new();
        let c0 = st.read("Coals");
        let n0 = st.read("Newcastle");
        assert_eq!(st.print(c0), "Coals");
        assert_eq!(st.print(n0), "Newcastle");
    }

    #[test]
    fn parse_simple() {
        let mut st = SymTab::new();
        let gr: Grid<usize> = parse(&mut st, "Coals to Newcastle = 31337\n".as_bytes());
        assert_eq!(st.pname.len(), 2);
        assert_eq!(st.print(0), "Coals");
        assert_eq!(st.print(1), "Newcastle");
        assert_eq!(gr[0][1], Some(31337));
        assert_eq!(gr[1][0], Some(31337));
        assert_eq!(gr[0][0], None);
        assert_eq!(gr[0].len(), 2);
        assert_eq!(gr[1].len(), 1);
    }
}
