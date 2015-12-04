use std::collections::HashMap;
use std::convert::From;
use std::io::{stdin, BufRead};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct House {
    x: isize,
    y: isize,
}
impl House {
    fn meander_mut(&mut self, m: Move) {
        match m {
            Move::North => self.y -= 1,
            Move::South => self.y += 1,
            Move::East => self.x += 1,
            Move::West => self.x -= 1,
        }
    }
    fn meander(&self, m: Move) -> House {
        #![allow(dead_code)]
        let mut next = *self;
        next.meander_mut(m);
        next
    }
}

#[derive(Debug)]
enum Move {
    North,
    South,
    East,
    West,
}

impl From<char> for Move {
    fn from(c: char) -> Move {
        match c {
            '^' => Move::North,
            'v' => Move::South,
            '>' => Move::East,
            '<' => Move::West,
            _ => panic!("unexpected character {}", c)
        }
    }
}

fn houses(s: &str) -> usize {
    let mut presents: HashMap<House, usize> = HashMap::new();
    let mut santa = House { x: 0, y: 0 };
    presents.insert(santa, 1);
    for c in s.chars() {
        santa.meander_mut(Move::from(c));
        let presents_here = presents.entry(santa).or_insert(0);
        *presents_here += 1;
    }
    presents.len()
}

pub fn main() {
    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("I/O error reading stdin");
        let hs = houses(&line);
        println!("Santa visited {} house{}.", hs, if hs == 1 { "" } else { "s" });
    }
}

#[cfg(test)]
mod test {
    use super::{House, Move, houses};
    const H: House = House { x: 0, y: 0 };
    
    #[test]
    fn move_id() {
        assert_eq!(H.meander(Move::East).meander(Move::West), H);
        assert_eq!(H.meander(Move::North).meander(Move::South), H);
        assert_eq!(H.meander(Move::North).meander(Move::East).meander(Move::South),
                   H.meander(Move::East));
        assert_eq!(H.meander(Move::East).meander(Move::North).meander(Move::West),
                   H.meander(Move::North));
    }

    #[test]
    fn spec_houses() {
        assert_eq!(houses(">"), 2);
        assert_eq!(houses("^>v<"), 4);
        assert_eq!(houses("^v^v^v^v^v"), 2);
    }
}
