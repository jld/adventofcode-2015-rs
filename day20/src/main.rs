use std::collections::BinaryHeap;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::env;

type Num = usize;
type Address = Num;
type Payload = Num;

#[derive(Debug, PartialEq, Eq)]
struct Elf {
    house: Address,
    stride: Address,
}
impl Elf {
    fn new(x: Num) -> Self { Elf { house: x, stride: x } }
    fn next(&self) -> Self { Elf { house: self.house + self.stride, ..*self } }
    fn flat(&self) -> (Num, Num) { (self.house, self.stride) } // Sigh.
}
impl Ord for Elf {
    fn cmp(&self, other: &Elf) -> Ordering {
        other.flat().cmp(&self.flat()) // Sigh.
    }
}
impl PartialOrd for Elf {
    fn partial_cmp(&self, other: &Elf) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct ElfParade {
    pending: BinaryHeap<Elf>,
    here: Num,
    multiplier: Payload,
    ttl: Option<Num>,
}
impl ElfParade {
    fn new(mul: Payload, ret: Option<Num>) -> Self {
        ElfParade { pending: BinaryHeap::new(), here: 1, multiplier: mul, ttl: ret }
    }
}
impl Iterator for ElfParade {
    type Item = (Address, Payload);
    fn next(&mut self) -> Option<Self::Item> {
        let mut loot = 0;
        let here = self.here;
        self.pending.push(Elf::new(here));
        while self.pending.peek().map_or(false, |elf| elf.house == here) {
            let elf = self.pending.pop().unwrap();
            if self.ttl.map_or(true, |ttl| here < elf.stride * ttl) {
                self.pending.push(elf.next());
            }
            loot += elf.stride * self.multiplier;
        }
        self.here += 1;
        Some((here, loot))
    }
}

fn main() {
    let input: Num = env::args().nth(1).expect("supply puzzle input as first argument")
        .parse().unwrap();
    let (addr, _loot) = ElfParade::new(10, None).find(|&(_addr, loot)| loot >= input).unwrap();
    println!("Old rules: {}", addr);
    let (addr, _loot) = ElfParade::new(11, Some(50)).find(|&(_addr, loot)| loot >= input).unwrap();
    println!("New rules: {}", addr);
}

#[cfg(test)]
mod tests {
    use super::ElfParade;

    #[test]
    fn example_nth() {
        assert_eq!(ElfParade::new(10, None).nth(0).unwrap(), (1, 10));
        assert_eq!(ElfParade::new(10, None).nth(3).unwrap(), (4, 70));
    }

    #[test]
    fn example_sums() {
        let (addrs, loot): (Vec<_>, Vec<_>) = ElfParade::new(10, None).take(9).unzip();
        assert_eq!(addrs, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(loot, vec![10, 30, 40, 70, 60, 120, 80, 150, 130]);
    }

    #[test]
    fn with_ttl_1() {
        let (_, loot): (Vec<_>, Vec<_>) = ElfParade::new(10, Some(8)).take(9).unzip();
        assert_eq!(loot, vec![10, 30, 40, 70, 60, 120, 80, 150, 120]);
    }
    #[test]
    fn with_ttl_2() {
        let (_, loot): (Vec<_>, Vec<_>) = ElfParade::new(10, Some(3)).take(9).unzip();
        assert_eq!(loot, vec![10, 30, 40, 60, 50, 110, 70, 120, 120]);
    }
    #[test]
    fn with_ttl_3() {
        // A test dev walks into a bar and orders -1 beers....
        let (_, loot): (Vec<_>, Vec<_>) = ElfParade::new(10, Some(1)).take(9).unzip();
        assert_eq!(loot, vec![10, 20, 30, 40, 50, 60, 70, 80, 90]);
    }
}
