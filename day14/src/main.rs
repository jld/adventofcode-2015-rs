extern crate regex;
extern crate util;
mod types;
mod parser;

use std::cmp::min;
use std::env;
use std::str::FromStr;
use std::io::stdin;

use parser::Parser;
use types::{Speed,Time,Dist,Points};

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
// I started to write a ReindeerIter for part 2, but then realized
// that having a simple closed form for position vs. time like that
// makes it not really pointful.

fn winner<'r, Score, Rules>(rules: Rules, deer: &'r [Reindeer], t: Time)
                            -> (Score, Vec<&'r Reindeer>)
    where Score: Ord + Copy,
          Rules: FnOnce(&'r [Reindeer], Time) -> Vec<Score> {
    let scores = rules(deer, t);
    assert_eq!(scores.len(), deer.len());
    let &best = scores.iter().max().expect("no deer");
    let winners = deer.iter().zip(scores).filter(|&(_,s)| s == best).map(|(d,_)| d).collect();
    (best, winners)
}

fn old_race(deer: &[Reindeer], t: Time) -> Vec<Dist> {
    deer.iter().map(|d| d.distance(t)).collect()
}

fn new_race(deer: &[Reindeer], t: Time) -> Vec<Points> {
    let mut scores = vec![Points(0); deer.len()];
    let times = (1..(t.0 + 1)).map(Time);
    for now in times {
        let places = old_race(deer, now);
        let &best = places.iter().max().expect("no deer");
        for (i, place) in places.into_iter().enumerate() {
            if place == best {
                scores[i] = scores[i] + Points(1);
            }
        }
    }
    scores
}

fn maybe_s(b: bool) -> &'static str { if b { "s" } else { "" } }

fn fmt_winner(deer: &[&Reindeer]) -> String {
    let names: Vec<_> = deer.iter().map(|d| &d.name as &str).collect();
    format!("{} win{}", names.join(" and "), maybe_s(deer.len() == 1))
}

fn main() {
    let p = Parser::new();
    let t = Time::from_str(&env::args().nth(1).unwrap()).unwrap();
    let stdin = stdin();
    let deer: Vec<_> = p.parse_all(stdin.lock()).into_iter().map(|r| r.unwrap()).collect();
    let (Dist(old_best), old_winner) = winner(old_race, &deer, t);
    println!("Old rules: {}, at {} km.",
             fmt_winner(&old_winner), old_best);
    let (Points(new_best), new_winner) = winner(new_race, &deer, t);
    println!("New rules: {}, with {} point{}.",
             fmt_winner(&new_winner), new_best, maybe_s(new_best != 1));
}

#[cfg(test)]
mod tests {
    use super::{Reindeer, old_race, new_race, winner};
    use types::{Num,Speed,Time,Dist,Points};

    pub fn rd(n: &str, s: Num, b: Num, r: Num) -> Reindeer {
        Reindeer { name: n.to_owned(), speed: Speed(s), burst: Time(b), rest: Time(r) }
    }

    fn comet() -> Reindeer { rd("Comet", 14, 10, 127) }
    fn dancer() -> Reindeer { rd("Dancer", 16, 11, 162) }
    fn blixem() -> Reindeer { rd("Blixem", 20, 7, 130) }

    #[test]
    fn examples() {
        assert_eq!(comet().distance(Time(1000)), Dist(1120));
        assert_eq!(dancer().distance(Time(1000)), Dist(1056));
    }

    #[test]
    fn as_old_race() {
        assert_eq!(old_race(&[comet(), dancer()], Time(1000)),
                   vec![Dist(1120), Dist(1056)]);
        assert_eq!(winner(old_race, &[comet(), dancer()], Time(1000)),
                   (Dist(1120), vec![&comet()]));
    }

    #[test]
    fn special_case() {
        assert_eq!(comet().distance(Time(5)), Dist(14 * 5));
        assert_eq!(comet().distance(Time(10)), Dist(14 * 10));
        assert_eq!(comet().distance(Time(10 + 127)), Dist(14 * 10));
        assert_eq!(comet().distance(Time(10 + 128)), Dist(14 * 11));
    }

    #[test]
    fn as_new_race() {
        assert_eq!(new_race(&[comet(), dancer()], Time(1000)),
                   vec![Points(312), Points(689)]);
        assert_eq!(winner(new_race, &[comet(), dancer()], Time(1000)),
                   (Points(689), vec![&dancer()]));
    }

    #[test]
    fn tied() {
        let exp = Dist(140 * (1000 / 137 + 1));
        assert_eq!(old_race(&[comet(), blixem()], Time(1000)),
                   vec![exp, exp]);
        assert_eq!(winner(old_race, &[comet(), blixem()], Time(1000)),
                   (exp, vec![&comet(), &blixem()]));
    }
}
