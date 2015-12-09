mod reader;

use std::io::{stdin, BufRead};
use std::mem::size_of;

type Dist = usize;
type Grid = reader::Grid<Dist>;
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
    fn push(&mut self, i: usize) -> bool {
        if self.mask & 1 << i == 0 {
            self.stack.push(i as u8);
            self.mask |= 1 << i;
            true
        } else {
            false
        }
    }
    fn top(&self) -> usize {
        self.stack[self.stack.len() - 1] as usize
    }
    fn pop(&mut self) {
        let i = self.stack.pop().expect("stack underflow") as usize;
        debug_assert!(self.contains(i));
        self.mask &= !(1 << i);
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
    let i = st.top();
    for (j, od) in g[i].iter().enumerate() {
        if let Some(d) = *od {
            if !st.contains(j) {
                st.push(j);
                search(g, st, be, so_far + d);
                debug_assert!(st.top() == j);
                st.pop();
            }
        }
    }
}

fn compute<B: BufRead, C: Cmp>(input: B, cmp: C) -> (Dist, Vec<String>) {
    let mut stab = reader::SymTab::new();
    let g: reader::Grid<usize> = reader::parse(&mut stab, input);

    let mut st = State::new(g.len());
    let mut be = Best::new(cmp);
    for i in 0..g.len() {
        debug_assert!(st.len() == 0);
        st.push(i);
        search(&g, &mut st, &mut be, 0);
        st.pop();
    }
    (be.dist.expect("No path!?"), be.path.iter().map(|i| stab.print(*i as usize)).collect())
}

pub fn main() {
    let stdin = stdin();
    let (dist, places) = compute(stdin.lock(), Shortest);
    println!("Path: {}", places.join(" -> "));
    println!("Length: {}", dist);
}

#[cfg(test)]
mod test {
    use super::{compute,Shortest,Longest};

    const EXAMPLE: &'static str = "\
        London to Dublin = 464\n\
        London to Belfast = 518\n\
        Dublin to Belfast = 141\n";

    #[test]
    fn example() {
        let (dist, path) = compute(EXAMPLE.as_bytes(), Shortest);
        assert_eq!(dist, 605);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "London");
        assert_eq!(path[1], "Dublin");
        assert_eq!(path[2], "Belfast");
    }

    #[test]
    fn example_long() {
        let (dist, path) = compute(EXAMPLE.as_bytes(), Longest);
        assert_eq!(dist, 982);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "Dublin");
        assert_eq!(path[1], "London");
        assert_eq!(path[2], "Belfast");
    }
}
