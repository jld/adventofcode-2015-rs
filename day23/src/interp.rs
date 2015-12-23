use std::borrow::Borrow;
use std::convert::From;
use std::default::Default;
use std::iter::{FromIterator,IntoIterator};
use std::isize;
use std::ops::{Index,IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    A = 0,
    B = 1,
}
pub const REGS: usize = 2;

pub type Offset = isize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insn {
    Hlf(Reg),
    Tpl(Reg),
    Inc(Reg),
    Jmp(Offset),
    Jie(Reg, Offset),
    Jio(Reg, Offset)
}

pub type Nat = u64;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegFile([Nat; REGS]);
impl Default for RegFile {
    fn default() -> RegFile {
        RegFile([0; REGS])
    }
}
impl Index<Reg> for RegFile {
    type Output = Nat;
    fn index(&self, r: Reg) -> &Nat {
        &self.0[r as usize]
    }
}
impl IndexMut<Reg> for RegFile {
    fn index_mut(&mut self, r: Reg) -> &mut Nat {
        &mut self.0[r as usize]
    }
}
impl FromIterator<(Reg, Nat)> for RegFile {
    fn from_iter<It>(it: It) -> Self where It: IntoIterator<Item=(Reg, Nat)> {
        let mut that: Self = Default::default();
        for (k, v) in it {
            that[k] = v;
        }
        that
    }
}
// Kind of hacky
impl<B: Borrow<[(Reg, Nat)]>> From<B> for RegFile {
    fn from(other: B) -> Self {
        Self::from_iter(other.borrow().iter().cloned())
    }
}

fn halve(n: Nat) -> Nat {
    assert!(n % 2 == 0, "Halving an odd number is underspecified.");
    n / 2
}

pub fn run(prog: &[Insn], init: &RegFile) -> RegFile {
    let mut regs = init.clone();
    let mut pc: isize = 0;
    assert!(prog.len() < isize::MAX as usize); // Trivially true, but still.
    while pc >= 0 && pc < prog.len() as isize {
        let mut jump = 1;
        match prog[pc as usize] {
            Insn::Hlf(r) => regs[r] = halve(regs[r]),
            Insn::Tpl(r) => regs[r] = regs[r].checked_mul(3).expect("overflow in `tpl`"),
            Insn::Inc(r) => regs[r] = regs[r].checked_add(1).expect("overflow in `inc`"),
            Insn::Jmp(off) => jump = off,
            Insn::Jie(r, off) => if regs[r] % 2 == 0 { jump = off },
            Insn::Jio(r, off) => if regs[r] == 1 { jump = off },
        };
        pc = pc.saturating_add(jump);
    }
    regs
}

#[cfg(test)]
mod tests {
    use std::convert::Into;
    use super::{Insn,Reg,run};

    #[test]
    fn example() {
        assert_eq!(run(&[Insn::Inc(Reg::A),
                         Insn::Jio(Reg::A, 2),
                         Insn::Tpl(Reg::A),
                         Insn::Inc(Reg::A)], &[].into()),
                   [(Reg::A, 2)].into());
    }
    
    #[test]
    fn not_example() {
        assert_eq!(run(&[Insn::Inc(Reg::A),
                         Insn::Jie(Reg::A, 2),
                         Insn::Tpl(Reg::A),
                         Insn::Inc(Reg::A)], &[].into()),
                   [(Reg::A, 4)].into());
        assert_eq!(run(&[Insn::Inc(Reg::A),
                         Insn::Tpl(Reg::A),
                         Insn::Jio(Reg::A, 2),
                         Insn::Tpl(Reg::A),
                         Insn::Inc(Reg::A)], &[].into()),
                   [(Reg::A, 10)].into());
    }

    #[test]
    fn collatz() {
        let start = 27;
        let steps = 111;
        assert_eq!(run(&[Insn::Jio(Reg::A, -1),
                         Insn::Inc(Reg::B),
                         Insn::Jie(Reg::A, 4),
                         Insn::Tpl(Reg::A),
                         Insn::Inc(Reg::A),
                         Insn::Jmp(-5),
                         Insn::Hlf(Reg::A),
                         Insn::Jmp(-2)],
                       &[(Reg::A, start)].into()),
                   [(Reg::A, 1),
                    (Reg::B, steps)].into());
    }
}
                   
