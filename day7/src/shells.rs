use ast::{Signal,Gate};
use eager::{Eager,CheckedEager,EagerError,NoError};
use lazy::{Lazy,UnsafeLazy,LazyError};
use generic::{Linker,LinkerError,Eval,Program,ProgramT,Strategy,Decl};

pub type Insn = (Gate<String>, String);

macro_rules! make_eval { {$name:ident<$Hkt:ident> -> $Err:ty} => {
    pub fn $name(insns: Vec<Insn>, outputs: &[&str])
                 -> Result<Vec<Signal>, Error<$Err>> {
        gen_eval($Hkt, insns, outputs)
    }
}}

type Prog = Program<Gate<Decl>, String>;

fn gen_eval<S: for<'p> Strategy<'p, Prog>>(strat: S, insns: Vec<Insn>, outputs: &[&str])
                                           -> Result<Vec<Signal>, Error<S::Error>> {
    let mut ld = Linker::new();
    for (gate, out) in insns {
        try!(ld.define(&out, gate));
    }
    let prog = try!(ld.link(outputs));
    let eval = strat.load(&prog);
    let mut sigs = Vec::new();
    for &entry in prog.entries() {
        sigs.push(try!(eval.run(entry).map_err(Error::EvalError)));
    }
    Ok(sigs)
}

make_eval!{eval_eager<Eager> -> NoError}
make_eval!{eval_eager_checked<CheckedEager> -> EagerError<String>}
make_eval!{eval_lazy<Lazy> -> LazyError<String>}
make_eval!{eval_lazy_unsafe<UnsafeLazy> -> LazyError<String>}

#[derive(Debug)]
pub enum Error<EvalError> {
    EvalError(EvalError),
    LinkerError(LinkerError<String>),
}
impl<E> From<LinkerError<String>> for Error<E> {
    fn from(e: LinkerError<String>) -> Self { Error::LinkerError(e) }
}

#[cfg(test)]
mod test {
    use super::{eval_eager, eval_eager_checked, eval_lazy, eval_lazy_unsafe};
    use ast::{Gate};

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

    #[test]
    fn checked_yes() {
        assert_eq!(eval_eager_checked(vec![(Gate::Imm(0x0123), s("a")),
                                           (Gate::Not(s("a")), s("b"))], &["b"]).unwrap(),
                   vec![0xfedc]);
        assert_eq!(eval_eager_checked(vec![(Gate::Not(s("a")), s("b")),
                                           (Gate::Imm(0x0123), s("a"))], &["b"]).unwrap(),
                   vec![0xfedc]);
    }

    #[test]
    fn lazy_yes() {
        assert_eq!(eval_lazy(vec![(Gate::Imm(0x0123), s("a")),
                                           (Gate::Not(s("a")), s("b"))], &["b"]).unwrap(),
                   vec![0xfedc]);
        assert_eq!(eval_lazy(vec![(Gate::Not(s("a")), s("b")),
                                           (Gate::Imm(0x0123), s("a"))], &["b"]).unwrap(),
                   vec![0xfedc]);
    }

    #[test]
    fn lazy_unsafe_yes() {
        assert_eq!(eval_lazy_unsafe(vec![(Gate::Imm(0x0123), s("a")),
                                         (Gate::Not(s("a")), s("b"))], &["b"]).unwrap(),
                   vec![0xfedc]);
        assert_eq!(eval_lazy_unsafe(vec![(Gate::Not(s("a")), s("b")),
                                         (Gate::Imm(0x0123), s("a"))], &["b"]).unwrap(),
                   vec![0xfedc]);
    }

    // TODO/FIXME/XXX/etc.: Write some tests for the error cases.  I
    // wrote that code, so it might as well do something.  (Except the
    // error case for the unchecked eager evaluator hitting a cycle,
    // because that's a stack overflow and I think those just kill
    // everything.)
}
