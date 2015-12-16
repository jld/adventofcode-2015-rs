extern crate util;
mod parser;
mod search;

use std::cmp::max;
use std::convert::From;
use std::io::{stdin,BufRead};
use std::ops::{Add, Mul};

use parser::{parse,Error};
use search::exhaustive;

pub type Num = i64;
pub type Qty = u8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    pub capacity: Num,
    pub durability: Num,
    pub flavor: Num,
    pub texture: Num,
    pub calories: Num,
}
impl Stats {
    fn zero() -> Stats {
        Stats { capacity: 0, durability: 0, flavor: 0, texture: 0, calories: 0 }
    }
    fn eval(&self) -> Num {
        max(self.capacity, 0) *
            max(self.durability, 0) *
            max(self.flavor, 0) *
            max(self.texture, 0)
    }
}
impl Add<Stats> for Stats {
    type Output = Stats;
    fn add(self, other: Stats) -> Stats {
        Stats {
            // There ought to be a way to use macros here...
            capacity: self.capacity + other.capacity,
            durability: self.durability + other.durability,
            flavor: self.flavor + other.flavor,
            texture: self.texture + other.texture,
            calories: self.calories + other.calories,
        }
    }
}
impl<N: Copy> Mul<N> for Stats where Num: From<N> {
    type Output = Stats;
    fn mul(self, other: N) -> Stats {
        let n = Num::from(other);
        Stats {
            capacity: self.capacity * n,
            durability: self.durability * n,
            flavor: self.flavor * n,
            texture: self.texture * n,
            calories: self.calories * n,            
        }
    }
}

fn main() {
    let stdin = stdin();
    let mut names = Vec::new();
    let mut ingrs = Vec::new();
    for (ln, line) in stdin.lock().lines().enumerate() {
        let line = line.expect("I/O error");
        match parse(&line) {
            Err(Error::NumberFail) => panic!("line {}: bad number", ln),
            Err(Error::SpaceFail) => panic!("line {}: attribute must be name' 'value pair", ln),
            Err(Error::ColonFail) => panic!("line {}: ingredient must be name':'stats pair", ln),
            Err(Error::MissingStat(name)) => panic!("line {}: missing attribute {}", ln, name),
            Err(Error::DupStat(name, v0, v1)) => panic!("line {}: attribute {} given twice, \
                                                         as {} and {}", ln, name, v0, v1),
            Ok((name, ingr)) => {
                names.push(name.to_owned());
                ingrs.push(ingr);
            }
        }
    }
    let (score, soln) = search::exhaustive(&ingrs, 100);
    println!("Maximal cookie score: {}", score);
    for (i, name) in names.iter().enumerate() {
        println!("* {} tsp of {}", soln[i], name);
    }
}
