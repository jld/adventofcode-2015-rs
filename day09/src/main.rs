extern crate util;
use util::best::{Smallest,Largest,Cmp};

mod reader;

use std::io::{stdin, BufRead};

type Dist = usize;
type Grid = reader::Grid<Dist>;
type Problem = (reader::SymTab, Grid);
type State = util::StackSet<u32>;
type Best<C> = util::Best<Dist, Vec<u8>, C>;

fn search<C: Cmp<Dist>>(g: &Grid, st: &mut State, be: &mut Best<C>, so_far: Dist) {
    if st.is_full() {
        be.add(so_far, st as &[u8]);
        return;
    }
    // For C==Smallest this could prune if so_far >= be.dist.
    // For C==Largest, would need an estimate of largest possible rest-of-path.
    let i = *st.last().unwrap() as usize;
    for (j, od) in g[i].iter().enumerate() {
        if let Some(d) = *od {
            st.push(j as u8, |st| search(g, st, be, so_far + d));
        }
    }
}

fn parse<B: BufRead>(input: B) -> Problem {
    let mut stab = reader::SymTab::new();
    let g = reader::parse(&mut stab, input);
    (stab, g)
}

fn compute<C: Cmp<Dist>>(p: &Problem, cmp: C) -> (Dist, Vec<String>) {
    let &(ref stab, ref g) = p;
    let mut st = State::new(g.len()).expect("too many points");
    let mut be = Best::new(cmp);
    for i in 0..g.len() {
        st.push(i as u8, |st| search(&g, st, &mut be, 0));
    }
    let (dist, path) = be.expect("No path!?");
    (dist, path.iter().map(|i| stab.print(*i as usize)).collect())
}

pub fn main() {
    let stdin = stdin();
    let prob = parse(stdin.lock());
    let (sdist, splaces) = compute(&prob, Smallest);
    println!("Smallest: {}; {}", sdist, splaces.join(" -> "));
    let (ldist, lplaces) = compute(&prob, Largest);
    println!("Largest: {}; {}", ldist, lplaces.join(" -> "));
}

#[cfg(test)]
mod test {
    use util::best::{Smallest,Largest};
    use super::{compute,parse};

    const EXAMPLE: &'static str = "\
        London to Dublin = 464\n\
        London to Belfast = 518\n\
        Dublin to Belfast = 141\n";

    #[test]
    fn example() {
        let (dist, path) = compute(&parse(EXAMPLE.as_bytes()), Smallest);
        assert_eq!(dist, 605);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "London");
        assert_eq!(path[1], "Dublin");
        assert_eq!(path[2], "Belfast");
    }

    #[test]
    fn example_long() {
        let (dist, path) = compute(&parse(EXAMPLE.as_bytes()), Largest);
        assert_eq!(dist, 982);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "Dublin");
        assert_eq!(path[1], "London");
        assert_eq!(path[2], "Belfast");
    }
}
