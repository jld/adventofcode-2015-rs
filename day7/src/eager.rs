use generic::{Eval,Expr,Decl,ProgramT};

pub struct Eager<'p, P: ProgramT + 'p> {
    prog: &'p P
}
impl<'p, P: ProgramT + 'p> Eval<'p, P> for Eager<'p, P> {
    type Error = NoError;
    fn new(prog: &'p P) -> Eager<'p, P> { Eager { prog: prog } }
    fn run(&self, entry: Decl) -> Result<<P::Expr as Expr>::Value, NoError> {
        self.prog.lookup(entry).eval(|&pc| self.run(pc))
    }
}
#[derive(Debug)]
enum NoError { }

pub struct CheckedEager<'p, P: ProgramT + 'p> {
    prog: &'p P
}
impl<'p, P: ProgramT + 'p> Eval<'p, P> for CheckedEager<'p, P> {
    type Error = EagerError<P::OuterIdent>;
    fn new(prog: &'p P) -> CheckedEager<'p, P> { CheckedEager { prog: prog } }
    fn run(&self, entry: Decl) -> Result<<P::Expr as Expr>::Value, Self::Error> {
        self.run_with_limit(entry, self.prog.len())
    }
}
impl<'p, P: ProgramT> CheckedEager<'p, P> {
    fn run_with_limit(&self, pc: Decl, limit: usize)
                      -> Result<<P::Expr as Expr>::Value, EagerError<P::OuterIdent>> {
        if let Some(limit_dec) = limit.checked_sub(1) {
            self.prog.lookup(pc).eval(|&npc| self.run_with_limit(npc, limit_dec))
        } else {
            Err(EagerError::RecursionLimitExceeded(self.prog.debug(pc)))
        }
    }
}
#[derive(Debug)]
pub enum EagerError<Ident> {
    RecursionLimitExceeded(Ident)
}
