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

fn houses(s: &str, n: usize) -> usize {
    assert!(n >= 1);
    let mut presents: HashMap<House, usize> = HashMap::new();
    let mut santas = vec![House { x: 0, y: 0 }; n];
    presents.insert(santas[0], n); // spec: "(delivering two presents to the same starting house)"
    for (i, c) in s.chars().enumerate() {
        let santa = &mut santas[i % n];
        santa.meander_mut(Move::from(c));
        let presents_here = presents.entry(*santa).or_insert(0);
        *presents_here += 1;
    }
    // Okay, so nothing is using the per-house present counts, but whatever.
    presents.len()
}

// TODO: this could be a number-of-houses newtype with a stringification trait?
fn hprn(hs: usize) -> String {
    format!("{} house{}", hs, if hs == 1 { "" } else { "s" })
}

pub fn main() {
    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line = line.expect("I/O error reading stdin");
        let h1 = houses(&line, 1);
        let h2 = houses(&line, 2);
        println!("Santa alone would visit {}.", hprn(h1));
        println!("Santa with Robo-Santa would visit {}.", hprn(h2));
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

    // These macros are almost pointless, but why not.
    macro_rules! case_alone { ($s:expr => $h:expr) => { assert_eq!(houses($s, 1), $h) } }
    macro_rules! case_robo  { ($s:expr => $h:expr) => { assert_eq!(houses($s, 2), $h) } }

    #[test]
    fn spec_alone() {
        case_alone!(">" => 2);
        case_alone!("^>v<" => 4);
        case_alone!("^v^v^v^v^v" => 2);
    }

    #[test]
    fn spec_robo() {
        case_robo!("^v" => 3);
        case_robo!("^>v<" => 3);
        case_robo!("^v^v^v^v^v" => 11);
    }
}
