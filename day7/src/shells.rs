use ast::{Signal,Gate};
use eager::{Eager,CheckedEager,EagerError};
use lazy::{Lazy,UnsafeLazy,LazyError};
use generic::{Linker,LinkerError,Eval,ProgramT};

pub type Insn = (Gate<String>, String);

pub fn eval_eager(insns: Vec<Insn>, outputs: &[&str])
                  -> Result<Vec<Signal>, LinkerError<String>> {
    let mut ld = Linker::new();
    for (gate, out) in insns {
        try!(ld.define(&out, gate));
    }
    let prog = try!(ld.link(outputs));
    let eval = Eager::new(&prog);
    Ok(prog.entries().iter().map(|&entry| eval.run(entry).unwrap()).collect())
}

#[cfg(test)]
mod test {
    use super::eval_eager;
    use ast::{Signal,Gate,Shift};

    fn s(s: &str) -> String { s.to_owned() }

    #[test]
    fn eager_simple() {
        assert_eq!(eval_eager(vec![(Gate::Imm(123), s("x"))], &["x"]).unwrap(), vec![123]);
        assert_eq!(eval_eager(vec![(Gate::Imm(0x0123), s("a")),
                                   (Gate::Not(s("a")), s("b"))], &["b"]).unwrap(), vec![0xfedc]);
        assert_eq!(eval_eager(vec![(Gate::Not(s("a")), s("b")),
                                   (Gate::Imm(0x0123), s("a"))], &["b"]).unwrap(), vec![0xfedc]);
    }

    #[test]
    fn eager_example() {
        assert_eq!(eval_eager(vec![(Gate::Imm(123), s("x")),
                                   (Gate::Imm(456), s("y")),
                                   (Gate::And(s("x"), s("y")), s("d")),
                                   (Gate::Or(s("x"), s("y")), s("e")),
                                   (Gate::lshift(s("x"), 2), s("f")),
                                   (Gate::rshift(s("y"), 2), s("g")),
                                   (Gate::Not(s("x")), s("h")),
                                   (Gate::Not(s("y")), s("i"))],
                              &["d", "e", "f", "g", "h", "i", "x", "y"]).unwrap(),
                   vec![72, 507, 492, 114, 65412, 65079, 123, 456]);
    }
}
