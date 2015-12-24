extern crate util;
use std::cell::Cell;
use std::cmp::Ord;
use std::env;
use std::io::{stdin,BufRead};
use util::SubsetSumIter;

type Weight = u16;
type Entangle = u64;
type Bag = Vec<Weight>;

#[derive(Debug, PartialEq, Eq)]
struct Plan {
    front: Bag,
    back: Vec<Bag>,
    ent: Entangle,
}

impl Plan {
    fn new(front: Bag, back: Vec<Bag>) -> Self {
        let ent = front.iter().fold(1 as Entangle, |e, &w| e.checked_mul(w as Entangle)
                                      .expect("quantum entanglement overflow"));
        Plan {
            front: front,
            back: back,
            ent: ent,
        }
    }

    fn badness(&self) -> (usize, Entangle) {
        (self.front.len(), self.ent)
    }
    fn is_better(&self, other: &Self) -> bool {
        self.badness() < other.badness()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Fail {
    NonDiv,
    Unsat
}

// All that matters for the non-front groups is that there exist *some* equal split.
fn whatever(stuff: Bag, groups: usize, each: Weight) -> Option<Vec<Bag>> {
    if groups == 0 {
        return Some(vec![]);
    }
    let mut split = SubsetSumIter::new(&stuff, each);
    split.next().and_then(|(mine, theirs)| {
        whatever(theirs, groups - 1, each).map(|mut rest| {
            rest.push(mine);
            rest
        })
    })
}

fn compute(mut stuff: Bag, groups: usize) -> Result<Plan, Fail> {
    let total = stuff.iter().fold(0 as Weight, |a, &i| a.checked_add(i).expect("weight overflow"));
    if total % groups as Weight != 0 { return Err(Fail::NonDiv) }
    let each = total / groups as Weight;

    stuff.sort_by(|a, b| b.cmp(a)); // Bias towards the larger elements in front.
    let thirds = SubsetSumIter::new(&stuff, each);

    let fewest = Cell::new(stuff.len() + 1);
    let front_candidates = thirds.filter(|&(ref front, ref _back)| {
        front.len() <= fewest.get() && { fewest.set(front.len()); true }
    });

    let balanced = front_candidates.filter_map(|(front, back)| {
        whatever(back, groups - 1, each).map(|back| Plan::new(front, back))
    });
    balanced.fold(Err(Fail::Unsat), |acc, plan| {
        match acc {
            Err(_) => Ok(plan),
            Ok(prev) => Ok(if plan.is_better(&prev) { plan } else { prev })
        }
    })
}

fn main() {
    let n: usize = env::args().nth(1).unwrap_or_else(|| "3".to_owned()).parse().expect("NaN");
    assert!(n > 0);

    let stdin = stdin();
    let stuff: Bag =
        stdin.lock().lines().map(|l| l.expect("I/O error").parse().expect("NaN")).collect();

    match compute(stuff, n) {
        Err(Fail::NonDiv) => println!("Total weight not divisible by {}!", n),
        Err(Fail::Unsat) => println!("Packages can't be divided evenly!"),
        Ok(plan) => {
            println!("Front: {:?}", plan.front);
            for (i, name) in [" Left", "Right", "Trunk"].iter().enumerate() {
                if let Some(thing) = plan.back.get(i) {
                    println!("{}: {:?}", name, thing);
                }
            }
            for (i, thing) in plan.back.iter().skip(3).enumerate() {
                println!("Other Place #{}: {:?}", i+1, thing);
            }
            println!("-- ");
            println!("Quantum Entanglement: {}", plan.ent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compute, Plan, Fail};

    #[test]
    fn fails() {
        assert_eq!(compute(vec![1, 1, 3], 3), Err(Fail::NonDiv));
        assert_eq!(compute(vec![1, 1, 4], 3), Err(Fail::Unsat));
        assert_eq!(compute(vec![1, 2, 3], 3), Err(Fail::Unsat));
        assert_eq!(compute(vec![1, 2, 3], 3), Err(Fail::Unsat));
        assert_eq!(compute(vec![2, 2, 2, 3], 3), Err(Fail::Unsat));
    }

    #[test]
    fn simple() {
        assert_eq!(compute(vec![3, 3, 3], 3), Ok(Plan {
            front: vec![3],
            back: vec![vec![3], vec![3]],
            ent: 3
        }));
        assert_eq!(compute(vec![2, 3, 2, 3, 2, 3], 3), Ok(Plan {
            front: vec![3, 2],
            back: vec![vec![3, 2], vec![3, 2]],
            ent: 6
        }));
    }

    #[test]
    fn example() {
        let plan = compute(vec![1, 2, 3, 4, 5, 7, 8, 9, 10, 11], 3).unwrap();
        assert_eq!(plan.ent, 99);
        assert_eq!(plan.front, &[11, 9]);
        // The problem-setter's combination enumerator seems to go
        // left-to-right and with-then-without; mine is right-to-left
        // and without-then-with, so it doesn't match.
    }
}
