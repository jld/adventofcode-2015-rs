use regex::Regex;
use types::{Speed,Time};
use ::Reindeer;

use std::io;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    SyntaxError,
    IntError(ParseIntError),
    IOError(io::Error),
}
// Sigh.
macro_rules! impl_from{{ $outer:ident: $($v:ident($inner:ty)),* } => {
    $(impl From<$inner> for $outer { fn from(x: $inner) -> $outer { $outer::$v(x) }})*
}}
impl_from!{ Error: IntError(ParseIntError), IOError(io::Error) }

pub struct Parser(Regex);
impl Parser {
    pub fn new() -> Parser {
        Parser(Regex::new(r"(?P<name>\S+) can fly (?P<speed>\d+) km/s for (?P<burst>\d+) (?x)
                       (?-x)seconds?, but then must rest for (?P<rest>\d+) seconds?\.").unwrap())
    }
    pub fn parse(&self, s: &str) -> Result<Reindeer,Error> {
        let caps = try!(self.0.captures(s).ok_or(Error::SyntaxError));
        let name = caps.name("name").unwrap().to_owned();
        let speed = try!(Speed::from_str(caps.name("speed").unwrap()));
        let burst = try!(Time::from_str(caps.name("burst").unwrap()));
        let rest = try!(Time::from_str(caps.name("rest").unwrap()));
        Ok(Reindeer{ name: name, speed: speed, burst: burst, rest: rest })
    }
    pub fn parse_all<B: io::BufRead>(&self, b: B) -> Vec<Result<Reindeer,Error>> {
        b.lines().map(|line| {
            let line = try!(line);
            self.parse(&line)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use ::tests::rd;

    #[test]
    fn examples() {
        let p = Parser::new();
        let comet = "Comet can fly 14 km/s for 10 seconds, but then must rest for 127 seconds.";
        let dancer = "Dancer can fly 16 km/s for 11 seconds, but then must rest for 162 seconds.";
        assert_eq!(p.parse(comet).unwrap(), rd("Comet", 14, 10, 127));
        assert_eq!(p.parse(dancer).unwrap(), rd("Dancer", 16, 11, 162));
    }
}
