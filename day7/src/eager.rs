use generic::{Eval,Env,Expr,Decl,ProgramT};

#[derive(Debug)]
enum NoError { }

pub struct Eager;
impl<P: ProgramT> Eval<P> for Eager {
    type Error = NoError;
    fn run(&self, prog: &P) -> Result<<P::Expr as Expr>::Value, NoError> {
        eager_eval(prog, prog.entry())
    }
}
fn eager_eval<P: ProgramT>(prog: &P, pc: Decl) -> Result<<P::Expr as Expr>::Value, NoError> {
    prog.lookup(pc).eval(&EagerEnv{ prog: prog })
}
struct EagerEnv<'p, P: ProgramT + 'p> {
    prog: &'p P
}
impl<'p, P: ProgramT> Env for EagerEnv<'p, P> {
    type Ident = <P::Expr as Expr>::Ident;
    type Value = <P::Expr as Expr>::Value;
    type Error = NoError;
    fn get(&self, id: &Decl) -> Result<Self::Value, Self::Error> {
        eager_eval(self.prog, *id)
    }
}


pub struct CheckedEager;
impl<P: ProgramT> Eval<P> for CheckedEager {
    type Error = EagerError<P::OuterIdent>;
    fn run(&self, prog: &P) -> Result<<P::Expr as Expr>::Value, Self::Error> {
        checked_eager_eval(prog, prog.entry(), prog.len())
    }
}
fn checked_eager_eval<P: ProgramT>(prog: &P, pc: Decl, limit: usize)
                                   -> Result<<P::Expr as Expr>::Value,
                                             EagerError<P::OuterIdent>> {
    if let Some(limit) = limit.checked_sub(1) {
        prog.lookup(pc).eval(&CheckedEagerEnv{ prog: prog, limit: limit })
    } else {
        Err(EagerError::RecursionLimitExceeded(prog.debug(pc)))
    }
}
struct CheckedEagerEnv<'p, P: ProgramT + 'p> {
    prog: &'p P,
    limit: usize
}
impl<'p, P: ProgramT> Env for CheckedEagerEnv<'p, P> {
    type Ident = <P::Expr as Expr>::Ident;
    type Value = <P::Expr as Expr>::Value;
    type Error = EagerError<P::OuterIdent>;
    fn get(&self, id: &Decl) -> Result<Self::Value, Self::Error> {
        checked_eager_eval(self.prog, *id, self.limit)
    }
}
#[derive(Debug)]
pub enum EagerError<Ident> {
    RecursionLimitExceeded(Ident)
}
