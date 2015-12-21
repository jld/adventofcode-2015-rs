use std::collections::BinaryHeap;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::env;

type Num = usize;
type Address = Num;
type Payload = Num;

#[derive(Debug, PartialEq, Eq)]
struct Elf {
    house: Num,
    stride: Num,
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
}
impl ElfParade {
    fn new() -> Self {
        ElfParade { pending: BinaryHeap::new(), here: 1 }
    }
}
impl Iterator for ElfParade {
    type Item = (Address, Payload);
    fn next(&mut self) -> Option<Self::Item> {
        let mut loot = 0;
        let here = self.here;
        self.pending.push(Elf::new(here));
        while self.pending.peek().unwrap().house == here {
            let elf = self.pending.pop().unwrap();
            self.pending.push(elf.next());
            loot += elf.stride;
        }
        self.here += 1;
        Some((here, loot))
    }
}

fn main() {
    let input: Num = env::args().nth(1).expect("supply puzzle input as first argument")
        .parse().unwrap();
    let input = (input + 9) / 10; // http://tvtropes.org/pmwiki/pmwiki.php/Main/PinballScoring
    let (addr, _loot) = ElfParade::new().find(|&(_addr, loot)| loot >= input).unwrap();
    println!("{}", addr);
}

#[cfg(test)]
mod tests {
    use super::ElfParade;

    #[test]
    fn example_nth() {
        assert_eq!(ElfParade::new().nth(0).unwrap(), (1, 1));
        assert_eq!(ElfParade::new().nth(3).unwrap(), (4, 7));
    }

    #[test]
    fn example_sums() {
        let (addrs, loot): (Vec<_>, Vec<_>) = ElfParade::new().take(9).unzip();
        assert_eq!(addrs, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(loot, vec![1, 3, 4, 7, 6, 12, 8, 15, 13]);
    }
}
