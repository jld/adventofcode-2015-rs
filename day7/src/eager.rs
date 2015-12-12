use generic::{Eval,Expr,Decl,ProgramT,Strategy,ProgResult,Erroneous};

pub struct Eager;
impl<P: ProgramT> Erroneous<P> for Eager {
    type Error = NoError;
}
impl<'p, P: ProgramT + 'p> Strategy<'p, P> for Eager {
    type Eval = EagerEval<'p, P>;
    fn load(&self, prog: &'p P) -> Self::Eval {
        EagerEval { prog: prog }
    }
}
struct EagerEval<'p, P: ProgramT + 'p> {
    prog: &'p P
}
impl<'p, P: ProgramT + 'p> Eval<'p, P, NoError> for EagerEval<'p, P> {
    fn run(&self, entry: Decl) -> ProgResult<P, NoError> {
        self.prog.lookup(entry).eval(|&pc| self.run(pc))
    }
}
#[derive(Debug)]
pub enum NoError { }

pub struct CheckedEager;
impl<P: ProgramT> Erroneous<P> for CheckedEager {
    type Error = EagerError<P::OuterIdent>;
}
impl<'p, P: ProgramT + 'p> Strategy<'p, P> for CheckedEager {
    type Eval = CheckedEagerEval<'p, P>;
    fn load(&self, prog: &'p P) -> Self::Eval { CheckedEagerEval { prog: prog } }
}
struct CheckedEagerEval<'p, P: ProgramT + 'p> {
    prog: &'p P
}
impl<'p, P: ProgramT + 'p> Eval<'p, P, EagerError<P::OuterIdent>> for CheckedEagerEval<'p, P> {
    fn run(&self, entry: Decl) -> ProgResult<P, EagerError<P::OuterIdent>> {
        self.run_with_limit(entry, self.prog.len())
    }
}
impl<'p, P: ProgramT> CheckedEagerEval<'p, P> {
    fn run_with_limit(&self, pc: Decl, limit: usize) -> ProgResult<P, EagerError<P::OuterIdent>> {
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
