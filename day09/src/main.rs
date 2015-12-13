mod reader;

use std::io::{stdin, BufRead};
use std::mem::size_of;

type Dist = usize;
type Grid = reader::Grid<Dist>;
type Problem = (reader::SymTab, Grid);
type Mask = usize;

trait Cmp { fn better(&self, old: Dist, shiny: Dist) -> bool; }
struct Shortest;
impl Cmp for Shortest { fn better(&self, old: Dist, shiny: Dist) -> bool { shiny < old } }
struct Longest;
impl Cmp for Longest { fn better(&self, old: Dist, shiny: Dist) -> bool { shiny > old } }

struct State {
    stack: Vec<u8>,
    mask: Mask,
}
impl State {
    fn new(n: usize) -> State {
        assert!(n < size_of::<Mask>() * 8, "{} points is too many", n);
        State { stack: Vec::new(), mask: !0 << n }
    }
    fn push<'a, F>(&mut self, i: usize, f: F) where F: FnOnce(&mut State) {
        if !self.contains(i) {
            self.stack.push(i as u8);
            self.mask |= 1 << i;
            f(self);
            let _i = self.stack.pop().unwrap();
            debug_assert_eq!(_i as usize, i);
            self.mask &= !(1 << i);
        }
    }
    fn top(&self) -> usize {
        self.stack[self.stack.len() - 1] as usize
    }
    fn len(&self) -> usize {
        self.stack.len()
    }
    fn contains(&self, i: usize) -> bool {
        self.mask & 1 << i != 0
    }
    fn is_full(&self) -> bool {
        self.mask == !0
    }
}

struct Best<C: Cmp> {
    path: Vec<u8>,
    dist: Option<Dist>,
    cmp: C,
}
impl<C: Cmp> Best<C> {
    fn new(cmp: C) -> Best<C> { Best {
        path: Vec::new(),
        dist: None,
        cmp: cmp,
    }}
    fn add(&mut self, dist: Dist, path: &[u8]) {
        if self.dist.as_ref().map_or(true, |&old| self.cmp.better(old, dist)) {
            self.dist = Some(dist);
            self.path = path.to_owned();
        }
    }
}

fn search<C: Cmp>(g: &Grid, st: &mut State, be: &mut Best<C>, so_far: Dist) {
    if st.is_full() {
        be.add(so_far, &st.stack);
        return;
    }
    // For C==Shortest this could prune if so_far >= be.dist.
    // For C==Longest, would need an estimate of largest possible rest-of-path.
    let i = st.top();
    for (j, od) in g[i].iter().enumerate() {
        if let Some(d) = *od {
            st.push(j, |st| search(g, st, be, so_far + d));
        }
    }
}

fn parse<B: BufRead>(input: B) -> Problem {
    let mut stab = reader::SymTab::new();
    let g = reader::parse(&mut stab, input);
    (stab, g)
}

fn compute<C: Cmp>(p: &Problem, cmp: C) -> (Dist, Vec<String>) {
    let &(ref stab, ref g) = p;
    let mut st = State::new(g.len());
    let mut be = Best::new(cmp);
    for i in 0..g.len() {
        debug_assert!(st.len() == 0);
        debug_assert!(!st.contains(i));
        st.push(i, |st| search(&g, st, &mut be, 0));
    }
    (be.dist.expect("No path!?"), be.path.iter().map(|i| stab.print(*i as usize)).collect())
}

pub fn main() {
    let stdin = stdin();
    let prob = parse(stdin.lock());
    let (sdist, splaces) = compute(&prob, Shortest);
    println!("Shortest: {}; {}", sdist, splaces.join(" -> "));
    let (ldist, lplaces) = compute(&prob, Longest);
    println!("Longest: {}; {}", ldist, lplaces.join(" -> "));
}

#[cfg(test)]
mod test {
    use super::{compute,parse,Shortest,Longest};

    const EXAMPLE: &'static str = "\
        London to Dublin = 464\n\
        London to Belfast = 518\n\
        Dublin to Belfast = 141\n";

    #[test]
    fn example() {
        let (dist, path) = compute(&parse(EXAMPLE.as_bytes()), Shortest);
        assert_eq!(dist, 605);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "London");
        assert_eq!(path[1], "Dublin");
        assert_eq!(path[2], "Belfast");
    }

    #[test]
    fn example_long() {
        let (dist, path) = compute(&parse(EXAMPLE.as_bytes()), Longest);
        assert_eq!(dist, 982);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "Dublin");
        assert_eq!(path[1], "London");
        assert_eq!(path[2], "Belfast");
    }
}