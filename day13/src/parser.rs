use regex::Regex;
use std::io::BufRead;
use std::num::ParseIntError;
use std::str::FromStr;
use util::symtab;
//use util::{SymTab, AutoVec};

pub type Points = i64;
pub type Grid = Box<[Box<[Points]>]>; // will be fully square

#[derive(Debug, PartialEq)]
struct Decl<Name> {
    subject: Name,
    object: Name,
    delta: Points
}

#[derive(Debug, PartialEq)]
pub enum Error {
    SyntaxError,
    IntError(ParseIntError),
}

pub struct Parser(Regex);
impl Parser {
    pub fn new() -> Parser {
        Parser(Regex::new(r"^(?P<subject>\S+) would (?P<verb>gain|lose) (?P<delta>\d+) happiness units by sitting next to (?P<object>\S+)\.$").unwrap())
    }
    fn parse_line<'l>(&self, line: &'l str) -> Result<Decl<&'l str>, Error> {
        let caps = try!(self.0.captures(line).ok_or(Error::SyntaxError));
        // Note: can't use `caps[_]`, because the `Index` traits have
        // `Output = str` instead of `Output = &'t str`, so using them
        // introduces an equality constraint between the lifetime of
        // the returned strings (including ones obtained from other
        // methods) and that of `caps` itself, which means they can
        // outlive it in order to be returned.  The error message for
        // that is...  not maximally clear.
        let delta = try!(Points::from_str(caps.name("delta").unwrap()).map_err(Error::IntError));
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
}

#[cfg(test)]
mod tests {
    use super::{Parser,Error,Decl};

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
                   Err(Error::SyntaxError));
        assert_eq!(p.parse_line("Alice would gain 54 happiness units by sitting next to Bob"),
                   Err(Error::SyntaxError));
        assert_eq!(p.parse_line("Alice would gain  54 happiness units by sitting next to Bob."),
                   Err(Error::SyntaxError));
        assert_eq!(p.parse_line("Alice would lose -54 happiness units by sitting next to Bob."),
                   Err(Error::SyntaxError));
        match p.parse_line("Alice would gain 9223372036854775808 happiness units by sitting \
                            next to Bob.") {
            Err(Error::IntError(_)) => (),
            other => panic!("expected Err(Error::IntError(_)), got {:?}", other)
        };
    }
}
