extern crate regex;
extern crate util;
mod types;
mod parser;

use std::cmp::min;
use std::env;
use std::str::FromStr;
use std::io::stdin;

use parser::Parser;
use types::{Speed,Time,Dist};

#[derive(PartialEq, Eq, Debug)]
pub struct Reindeer {
    pub name: String,
    pub speed: Speed,
    pub burst: Time,
    pub rest: Time,
}
impl Reindeer {
    fn distance(&self, t: Time) -> Dist {
        let cycletime = self.burst + self.rest;
        let cycles = t / cycletime;
        let partial = t % cycletime;
        let burn = cycles * self.burst + min(partial, self.burst);
        burn * self.speed
    }
}

fn main() {
    let p = Parser::new();
    let t = Time::from_str(&env::args().nth(1).unwrap()).unwrap();
    let stdin = stdin();
    let deer = p.parse_all(stdin.lock());
    let (best, winner) = deer.iter().map(|r| r.as_ref().unwrap())
        .map(|d| (d.distance(t), &d.name)).max()
        .expect("Can't have a race with no reindeer!");
    println!("{} wins, at {} km.", winner, best.0);
}

#[cfg(test)]
mod tests {
    use super::Reindeer;
    use types::{Num,Speed,Time,Dist};

    pub fn rd(n: &str, s: Num, b: Num, r: Num) -> Reindeer {
        Reindeer { name: n.to_owned(), speed: Speed(s), burst: Time(b), rest: Time(r) }
    }
    
    #[test]
    fn examples() {
        let comet = rd("Comet", 14, 10, 127);
        let dancer = rd("Dancer", 16, 11, 162);
        assert_eq!(comet.distance(Time(1000)), Dist(1120));
        assert_eq!(dancer.distance(Time(1000)), Dist(1056));
    }

    #[test]
    fn special_case() {
        let comet = rd("Comet", 14, 10, 127);
        assert_eq!(comet.distance(Time(5)), Dist(14 * 5));
        assert_eq!(comet.distance(Time(10)), Dist(14 * 10));
        assert_eq!(comet.distance(Time(10 + 127)), Dist(14 * 10));
        assert_eq!(comet.distance(Time(10 + 128)), Dist(14 * 11));
    }
        
}
