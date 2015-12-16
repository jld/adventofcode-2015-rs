extern crate util;
mod parser;
mod search;

use std::cmp::max;
use std::convert::From;
use std::ops::{Add, Mul};

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
    println!("Hello, world!");
}
