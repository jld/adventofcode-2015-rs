use std::collections::BinaryHeap;
use std::cmp::{PartialOrd, Ord, Ordering};
use std::env;

type Num = usize;
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
    type Item = Vec<Payload>;
    fn next(&mut self) -> Option<Self::Item> {
        let mut acc = vec![];
        self.pending.push(Elf::new(self.here));
        while self.pending.peek().unwrap().house == self.here {
            let elf = self.pending.pop().unwrap();
            self.pending.push(elf.next());
            acc.push(elf.stride);
        }
        self.here += 1;
        Some(acc)
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::ElfParade;

    #[test]
    fn example_nth() {
        assert_eq!(ElfParade::new().nth(0).unwrap(), vec![1]);
        assert_eq!(ElfParade::new().nth(3).unwrap(), vec![1, 2, 4]);
    }

    #[test]
    fn example_sums() {
        let loot = ElfParade::new().map(|es| es.into_iter().fold(0, |a, b| a + b));
        let loot: Vec<_> = loot.take(9).collect();
        assert_eq!(loot, vec![1, 3, 4, 7, 6, 12, 8, 15, 13]);
    }
}
