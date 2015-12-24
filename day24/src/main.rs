extern crate util;
use std::cell::Cell;
use std::cmp::{Ord,min};
use util::SubsetSumIter;

type Weight = u16;
type Count = u16;
type Entangle = u64;
type Bag = Vec<Weight>;

#[derive(Debug, PartialEq, Eq)]
struct Plan {
    bags: [Bag; 3],
    ent: Entangle
}

impl Plan {
    fn new(front: Bag, left: Bag, right: Bag) -> Self {
        let ent = front.iter().fold(1 as Entangle, |e, &w| e.checked_mul(w as Entangle)
                                    .expect("quantum entanglement overflow"));
        Plan {
            bags: [front, left, right],
            ent: ent
        }
    }

    fn front(&self) -> &Bag { &self.bags[0] }
    fn  left(&self) -> &Bag { &self.bags[1] }
    fn right(&self) -> &Bag { &self.bags[2] }

    fn badness(&self) -> (usize, Entangle) {
        (self.front().len(), self.ent)
    }
    fn is_better(&self, other: &Self) -> bool {
        self.badness() < other.badness()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Fail {
    NonDiv3,
    Unsat
}

fn compute(mut stuff: Vec<Weight>) -> Result<Plan, Fail> {
    let total = stuff.iter().fold(0 as Weight, |a, &i| a.checked_add(i).expect("weight overflow"));
    if total % 3 != 0 { return Err(Fail::NonDiv3) }
    stuff.sort_by(|a, b| b.cmp(a)); // Bias towards the larger elements in front.
    let thirds = SubsetSumIter::new(&stuff, total / 3);
    let fewest = Cell::new(stuff.len() + 1);
    let front_candidates = thirds.filter(|&(ref front, ref _back)| {
        front.len() <= fewest.get() && { fewest.set(front.len()); true }
    });
    let balanced = front_candidates.filter_map(|(front, back)| {
        let mut leftright = SubsetSumIter::new(&back, total / 3);
        // All that matters is that there exist *some* equal split.
        leftright.next().map(|(left, right)| Plan::new(front, left, right))
    });
    balanced.fold(Err(Fail::Unsat), |acc, plan| {
        match acc {
            Err(_) => Ok(plan),
            Ok(prev) => Ok(if plan.is_better(&prev) { plan } else { prev })
        }
    })
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::{compute, Plan, Fail};

    #[test]
    fn fails() {
        assert_eq!(compute(vec![1, 1, 3]), Err(Fail::NonDiv3));
        assert_eq!(compute(vec![1, 1, 4]), Err(Fail::Unsat));
        assert_eq!(compute(vec![1, 2, 3]), Err(Fail::Unsat));
        assert_eq!(compute(vec![1, 2, 3]), Err(Fail::Unsat));
        assert_eq!(compute(vec![2, 2, 2, 3]), Err(Fail::Unsat));
    }

    #[test]
    fn simple() {
        assert_eq!(compute(vec![3, 3, 3]), Ok(Plan {
            bags: [vec![3], vec![3], vec![3]],
            ent: 3
        }));
        assert_eq!(compute(vec![2, 3, 2, 3, 2, 3]), Ok(Plan {
            bags: [vec![3, 2], vec![3, 2], vec![3, 2]],
            ent: 6
        }));
    }

    #[test]
    fn example() {
        let plan = compute(vec![1, 2, 3, 4, 5, 7, 8, 9, 10, 11]).unwrap();
        assert_eq!(plan.ent, 99);
        assert_eq!(plan.front(), &[11, 9]);
        // The problem-setter's combination enumerator seems to go
        // left-to-right and with-then-without; mine is right-to-left
        // and without-then-with, so it doesn't match.
    }
}
