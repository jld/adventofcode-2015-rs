use regex::Regex;
use std::io;
use std::num::ParseIntError;
use std::result;
use std::str::FromStr;
use util::symtab;
use util::{SymTab, AutoVec};

pub type Points = i64;
pub type Grid = Box<[Box<[Points]>]>; // will be fully square
type GridAcc = AutoVec<AutoVec<Points>>;

fn finish_grid(acc: GridAcc) -> Grid {
    let n = acc.len();
    acc.into_iter().map(|row| {
        let row = row.into_boxed_slice(n);
        assert!(row.len() == n);
        row
    }).collect::<Vec<_>>().into_boxed_slice()
}

#[derive(Debug, PartialEq, Eq)]
struct Decl<Name> {
    subject: Name,
    object: Name,
    delta: Points
}
impl<'t> Decl<&'t str> {
    fn symbolize(&self, stab: &mut SymTab) -> Decl<symtab::Id> {
        Decl {
            subject: stab.read(self.subject),
            object: stab.read(self.object),
            // Basically `..*self` but without the type error:
            delta: self.delta,
        }
    }
}

#[derive(Debug)]
pub enum LineError {
    SyntaxError,
    BesideMyself,
    IntError(ParseIntError),
    IOError(io::Error)
}
impl PartialEq for LineError {
    // #[derive(PartialEq = "very partial")].  Sigh.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&LineError::SyntaxError, &LineError::SyntaxError) => true,
            (&LineError::BesideMyself, &LineError::BesideMyself) => true,
            // `ParseIntError: PartialEq`, but I don't care.
            _ => false
        }
    }
}
macro_rules! impl_from{{ $outer:ident: $($v:ident($inner:ty)),* } => {
    $(impl From<$inner> for $outer { fn from(x: $inner) -> $outer { $outer::$v(x) }})*
}}
impl_from!{ LineError: IntError(ParseIntError), IOError(io::Error) }

#[derive(Debug, PartialEq)]
pub struct Error {
    pub line: usize,
    pub what: LineError
}
type LineResult<T> = result::Result<T, LineError>;
pub type Result<T> = result::Result<T, Error>;

pub struct Parser(Regex);
impl Parser {
    pub fn new() -> Parser {
        Parser(Regex::new(r"^(?P<subject>\S+) would (?P<verb>gain|lose) (?P<delta>\d+) happiness units by sitting next to (?P<object>\S+)\.$").unwrap())
    }
    fn parse_line<'l>(&self, line: &'l str) -> LineResult<Decl<&'l str>> {
        let caps = try!(self.0.captures(line).ok_or(LineError::SyntaxError));
        // Note: can't use `caps[_]`, because the `Index` traits have
        // `Output = str` instead of `Output = &'t str`, so using them
        // introduces an equality constraint between the lifetime of
        // the returned strings (including ones obtained from other
        // methods) and that of `caps` itself, which means they can
        // outlive it in order to be returned.  The error message for
        // that is...  not maximally clear.
        //
        // (FIXME: what the above paragraph is actually only part of
        // the problem.)
        let delta = try!(Points::from_str(caps.name("delta").unwrap()));
        let delta = match caps.name("verb").unwrap() {
            "gain" => delta,
            "lose" => -delta,
            _ => unreachable!()
        };
        Ok(Decl {
            subject: caps.name("subject").unwrap(),
            object: caps.name("object").unwrap(),
            delta: delta
        })
    }
    fn parse_one(&self, stab: &mut SymTab, acc: &mut GridAcc,
                 line: io::Result<String>) -> LineResult<()> {
        let line = try!(line);
        let decl = try!(self.parse_line(&line)).symbolize(stab);
        if decl.subject == decl.object {
            return Err(LineError::BesideMyself);
        }
        *acc.at(decl.subject).at(decl.object) += decl.delta;
        *acc.at(decl.object).at(decl.subject) += decl.delta;
        Ok(())
    }
    pub fn parse<B: io::BufRead>(&self, b: B) -> Result<(SymTab, Grid)> {
        let mut stab = SymTab::new();
        let mut acc: GridAcc = AutoVec::new();
        for (ln, line) in b.lines().enumerate() {
            try!(self.parse_one(&mut stab, &mut acc, line).map_err(|err| {
                Error { line: ln + 1, what: err }
            }));
        }
        Ok((stab, finish_grid(acc)))
    }
}

#[cfg(test)]
mod tests {
    use util::{SymTab,AutoVec};
    use super::{Parser,Error,Decl,LineError,GridAcc,finish_grid};

    macro_rules! bxsl { [$($elem:expr),*] => { vec![$($elem),*].into_boxed_slice() } }

    #[test]
    fn lines() {
        let p = Parser::new();
        assert_eq!(p.parse_line("Alice would gain 54 happiness units by sitting next to Bob."),
                   Ok(Decl { subject: "Alice", object: "Bob", delta: 54 }));
        assert_eq!(p.parse_line("Bob would lose 7 happiness units by sitting next to Carol."),
                   Ok(Decl { subject: "Bob", object: "Carol", delta: -7 }));
    }

    #[test]
    fn failures() {
        let p = Parser::new();
        assert_eq!(p.parse_line("UALUEALUEALEUALE"),
                   Err(LineError::SyntaxError));
        assert_eq!(p.parse_line("Alice would gain 54 happiness units by sitting next to Bob"),
                   Err(LineError::SyntaxError));
        assert_eq!(p.parse_line("Alice would gain  54 happiness units by sitting next to Bob."),
                   Err(LineError::SyntaxError));
        assert_eq!(p.parse_line("Alice would lose -54 happiness units by sitting next to Bob."),
                   Err(LineError::SyntaxError));
        match p.parse_line("Alice would gain 9223372036854775808 happiness units by sitting \
                            next to Bob.") {
            Err(LineError::IntError(_)) => (),
            other => panic!("expected Err(LineError::IntError(_)), got {:?}", other)
        };
    }

    #[test]
    fn parse_one() {
        let p = Parser::new();
        let mut stab = SymTab::new();
        let mut acc: GridAcc = AutoVec::new();
        p.parse_one(&mut stab, &mut acc,
                    Ok("Alice would gain 54 happiness units by sitting next to Bob.".to_owned()))
            .unwrap();
        assert_eq!(stab.len(), 2);
        assert_eq!(stab.print(0), "Alice");
        assert_eq!(stab.print(1), "Bob");
        assert_eq!(finish_grid(acc), bxsl![bxsl![0, 54],
                                           bxsl![54, 0]]);
    }

    #[test]
    fn beside_self() {
        let p = Parser::new();
        let mut stab = SymTab::new();
        let mut acc: GridAcc = AutoVec::new();
        let r = p.parse_one(&mut stab, &mut acc, Ok("Alice would gain 54 happiness units by \
                                                     sitting next to Alice.".to_owned()));
        assert_eq!(r, Err(LineError::BesideMyself));
    }

    #[test]
    fn lined_error() {
        let p = Parser::new();
        let e = p.parse("\
            Alice would gain 54 happiness units by sitting next to Bob.\n\
            Alice would lose 79 happiness units by sitting next to Carol".as_bytes()).unwrap_err();
        assert_eq!(e, Error { line: 2, what: LineError::SyntaxError });
    }

    #[test]
    fn parse_two() {
        let p = Parser::new();
        let (stab, grid) = p.parse("\
            Alice would gain 54 happiness units by sitting next to Bob.\n\
            Alice would lose 79 happiness units by sitting next to Carol.\n\
            Carol would lose 62 happiness units by sitting next to Alice.".as_bytes()).unwrap();
        assert_eq!(stab.len(), 3);
        assert_eq!(stab.print(0), "Alice");
        assert_eq!(stab.print(1), "Bob");
        assert_eq!(stab.print(2), "Carol");
        assert_eq!(grid, bxsl![bxsl![0, 54, -79-62],
                               bxsl![54, 0, 0],
                               bxsl![-79-62, 0, 0]]);
    }
}
