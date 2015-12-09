mod reader;

use std::io::{stdin, BufRead};
use std::mem::size_of;

type Dist = usize;
type Grid = reader::Grid<Dist>;
type Mask = usize;

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

struct Best {
    path: Vec<u8>,
    dist: Dist,
}
impl Best {
    fn new() -> Best { Best {
        path: Vec::new(),
        dist: !0
    }}
    fn add(&mut self, dist: Dist, path: &[u8]) {
        if dist < self.dist {
            self.dist = dist;
            self.path = path.to_owned();
        }
    }
}

fn search(g: &Grid, st: &mut State, be: &mut Best, so_far: Dist) {
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

fn compute<B: BufRead>(b: B) -> (Dist, Vec<String>) {
    let mut stab = reader::SymTab::new();
    let g: reader::Grid<usize> = reader::parse(&mut stab, b);

    let mut st = State::new(g.len());
    let mut be = Best::new();
    for i in 0..g.len() {
        debug_assert!(st.len() == 0);
        st.push(i);
        search(&g, &mut st, &mut be, 0);
        st.pop();
    }
    assert!(be.dist != !0, "No path!?");
    (be.dist, be.path.iter().map(|i| stab.print(*i as usize)).collect())
}

pub fn main() {
    let stdin = stdin();
    let (dist, places) = compute(stdin.lock());
    println!("Path: {}", places.join(" -> "));
    println!("Length: {}", dist);
}

#[cfg(test)]
mod test {
    use super::compute;

    const EXAMPLE: &'static str = "\
        London to Dublin = 464\n\
        London to Belfast = 518\n\
        Dublin to Belfast = 141\n";

    #[test]
    fn example() {
        let (dist, path) = compute(EXAMPLE.as_bytes());
        assert_eq!(dist, 605);
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], "London");
        assert_eq!(path[1], "Dublin");
        assert_eq!(path[2], "Belfast");
    }
}
