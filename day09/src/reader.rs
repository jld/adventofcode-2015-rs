use std::fmt::Debug;
use std::io::{BufRead, Read};
use std::str::FromStr;

use util::{AutoVec,WriteOnce};

pub use util::SymTab; // FIXME
pub type Grid<N> = Box<[Box<[Option<N>]>]>;

type GridAcc<N> = AutoVec<AutoVec<WriteOnce<N>>>;

pub fn parse<N, B>(stab: &mut SymTab, b: B) -> Grid<N>
    where N: FromStr + Copy, N::Err: Debug, B: BufRead {
    let mut acc: GridAcc<N> = Default::default();
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
        acc.at(nsrc).at(ndst).set(dist).unwrap_or_else(|_v| {
            panic!("line {}: duplicate distance from {} to {}", ln, words[0], words[2]);
        });
        acc.at(ndst).at(nsrc).set(dist).unwrap_or_else(|_v| unreachable!());
    }
    acc.into_iter().map(|row| {
        row.into_iter().map(|cell| {
            cell.into_inner()
        }).collect::<Vec<_>>().into_boxed_slice()
    }).collect::<Vec<_>>().into_boxed_slice()
}

#[cfg(test)]
mod test {
    use super::{parse, Grid, SymTab};

    #[test]
    fn parse_simple() {
        let mut st = SymTab::new();
        let gr: Grid<usize> = parse(&mut st, "Coals to Newcastle = 31337\n".as_bytes());
        assert_eq!(st.len(), 2);
        assert_eq!(st.print(0), "Coals");
        assert_eq!(st.print(1), "Newcastle");
        assert_eq!(gr[0][1], Some(31337));
        assert_eq!(gr[1][0], Some(31337));
        assert_eq!(gr[0][0], None);
        assert_eq!(gr[0].len(), 2);
        assert_eq!(gr[1].len(), 1);
    }
}
