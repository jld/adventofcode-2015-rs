extern crate regex;
extern crate util;
mod parser;

use std::io::stdin;

use util::SymTab;
use util::best::Largest;
use parser::{Grid,Points,Parser};

type State = util::StackSet<u32>;
type Best = util::Best<Points, Vec<u8>, Largest>;
type Problem = (SymTab, Grid);

// There's still duplication with day09 here that could be factored out....
fn search(g: &Grid, st: &mut State, be: &mut Best, so_far: Points) {
    if st.is_full() {
        if st.len() < 3 {
            unimplemented!();
        }
        // (this part is different)
        let close = g[*st.last().unwrap() as usize][*st.first().unwrap() as usize];
        be.add(so_far + close, st as &[u8]);
    }
    let i = *st.last().unwrap() as usize;
    for (j, /* (also this) */ &d) in g[i].iter().enumerate() {
        st.push(j as u8, |st| search(g, st, be, so_far + d))
    }
}

// Duplication here too:
fn compute(prob: &Problem) -> (Points, Vec<String>) {
    let &(ref stab, ref grid) = prob;
    let mut st = State::new(grid.len()).expect("too many people");
    let mut be = Best::new(Largest);
    for i in 0..grid.len() {
        st.push(i as u8, |st| search(grid, st, &mut be, 0));
    }
    let (points, order) = be.unwrap();
    (points, order.iter().map(|i| stab.print(*i as usize)).collect())
}

pub fn main() {
    let p = Parser::new();
    let stdin = stdin();
    let prob = p.parse(stdin.lock()).unwrap();
    let (points, order) = compute(&prob);
    println!("Δhappiness = {}", points);
    println!("Order: {}.", order.join(", "));
}

#[cfg(test)]
mod tests {
    use super::compute;
    use parser::Parser;

    #[test]
    fn example() {
        let p = Parser::new();
        let text = "\
        Alice would gain 54 happiness units by sitting next to Bob.\n\
        Alice would lose 79 happiness units by sitting next to Carol.\n\
        Alice would lose 2 happiness units by sitting next to David.\n\
        Bob would gain 83 happiness units by sitting next to Alice.\n\
        Bob would lose 7 happiness units by sitting next to Carol.\n\
        Bob would lose 63 happiness units by sitting next to David.\n\
        Carol would lose 62 happiness units by sitting next to Alice.\n\
        Carol would gain 60 happiness units by sitting next to Bob.\n\
        Carol would gain 55 happiness units by sitting next to David.\n\
        David would gain 46 happiness units by sitting next to Alice.\n\
        David would lose 7 happiness units by sitting next to Bob.\n\
        David would gain 41 happiness units by sitting next to Carol.";
        let prob = p.parse(text.as_bytes()).unwrap();
        let (score, _order) = compute(&prob);
        assert_eq!(score, 330);
    }
}
