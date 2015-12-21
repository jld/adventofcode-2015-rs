use std::cmp::min;
use std::ops::Add;

// https://en.wikipedia.org/wiki/CYK_algorithm

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Sym(usize);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Cost(usize);
impl Add for Cost {
    type Output = Cost;
    fn add(self, other: Cost) -> Cost {
        Cost(self.0.saturating_add(other.0))
    }
}
impl Cost {
    fn nope() -> Cost {
        Cost(!0)
    }
    fn externalize(self) -> Option<usize> {
        if self == Self::nope() { None } else { Some(self.0) }
    }
}

pub struct CYK {
    syms: usize,
    next: Sym,
    prods: Vec<(Sym, [Sym; 2], Cost)>,
}
impl CYK {
    pub fn new(syms: usize) -> Self { CYK {
        syms: syms,
        next: Sym(syms),
        prods: Vec::new(),
    }}
    fn alloc_nt(&mut self) -> Sym {
        let rv = self.next;
        self.next.0 += 1;
        rv
    }
    pub fn add_rule(&mut self, lhs: usize, rhs: &[usize]) {
        assert!(lhs < self.syms, "CYK::add_rule: lhs out of bounds");
        for (i, &rh) in rhs.iter().enumerate() {
            assert!(rh < self.syms, "CYK::add_rule: rhs[{}] out of bounds", i);
        }
        assert!(rhs.len() >= 2, "CYK::add_rule: rhs.len() < 2 not implemented");
        let mut lhs = Sym(lhs);
        let mut rhs = rhs;
        while rhs.len() > 2 {
            let inter = self.alloc_nt();
            self.prods.push((lhs, [Sym(rhs[0]), inter], Cost(0)));
            lhs = inter;
            rhs = &rhs[1..];
        }
        self.prods.push((lhs, [Sym(rhs[0]), Sym(rhs[1])], Cost(1)));
    }
    pub fn solve(&self, starts: &[usize], sentence: &[usize]) -> Option<usize> {
        for (i, &start) in starts.iter().enumerate() {
            assert!(start < self.syms, "CYK::solve: starts[{}] out of bounds", i);
        }
        let mut state = Table::new(sentence.len(), self.next);
        for (i, &ltr) in sentence.iter().enumerate() {
            assert!(ltr < self.syms, "CYK::solve: sentence[{}] out of bounds", i);
            state.write(i, 1, Sym(ltr), Cost(0));
        }
        // Hey, maybe *not* naming the variables i, j, k will be clearer.
        for length in 2..(sentence.len() + 1) {
            for start in 0..(sentence.len() - length + 1) {
                for partition in 1..length {
                    for &(lhs, rhs, cost) in self.prods.iter() {
                        let newcost = cost +
                            state.read(start, partition, rhs[0]) + 
                            state.read(start + partition, length - partition, rhs[1]);
                        state.write(start, length, lhs, newcost);
                    }
                }
            }
        }
        starts.iter().map(|&start| state.read(0, sentence.len(), Sym(start)))
            .fold(Cost::nope(), |ca, cb| min(ca, cb)).externalize()
    }
}

struct Table {
    slen: usize,
    nsym: usize,
    stuff: Box<[Cost]>
}
impl Table {
    fn new(slen: usize, next: Sym) -> Table {
        let Sym(nsym) = next;
        let vec = vec![Cost::nope(); slen * slen * nsym];
        Table { slen: slen, nsym: nsym, stuff: vec.into_boxed_slice() }
    }
    fn get_index(&self, start: usize, len: usize, sym: Sym) -> usize {
        let Sym(sidx) = sym;
        debug_assert!(sidx < self.nsym);
        debug_assert!(start < self.slen);
        debug_assert!(start + len <= self.slen);
        // ...which means ~half the elements are unused.  "Fascinating."
        debug_assert!(len > 0);
        (((len - 1) * self.slen + start) * self.nsym) + sidx
    }
    fn read(&self, start: usize, len: usize, sym: Sym) -> Cost {
        self.stuff[self.get_index(start, len, sym)]
    }
    fn write(&mut self, start: usize, len: usize, sym: Sym, newcost: Cost) {
        let idx = self.get_index(start, len, sym);
        let newcost = min(self.stuff[idx], newcost);
        self.stuff[idx] = newcost;
    }
}

#[cfg(test)]
mod tests {
    use super::CYK;

    #[test]
    fn hohoho() {
        let mut cyk = CYK::new(2);
        cyk.add_rule(1, &[1, 0]);
        cyk.add_rule(1, &[0, 1]);
        cyk.add_rule(0, &[1, 1]);
        assert_eq!(cyk.solve(&[0, 1], &[1, 0, 1]), Some(2));
        assert_eq!(cyk.solve(&[0, 1], &[1, 0, 1, 0, 1, 0]), Some(5));
        assert_eq!(cyk.solve(&[0, 1], &[0, 0, 0]), None);
    }
}
