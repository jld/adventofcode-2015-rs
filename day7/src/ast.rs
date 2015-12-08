use generic::{Expr,ExprMap,Env};

pub type Signal = u16;
pub enum Gate<Ident> {
    Imm(Signal),
    Not(Ident),
    And(Ident, Ident),
    Or(Ident, Ident),
    LShift(Ident, Shift),
    RShift(Ident, Shift),
}
impl<Ident> Expr for Gate<Ident> {
    type Ident = Ident;
    type Value = Signal;
    fn eval<En>(&self, env: &En) -> Result<Self::Value, En::Error>
        where En: Env<Ident=Self::Ident, Value=Self::Value> {
        Ok(match *self {
            Gate::Imm(val) => val,
            Gate::Not(ref id) => { !try!(env.get(id)) },
            Gate::And(ref id0, ref id1) => try!(env.get(id0)) & try!(env.get(id1)),
            Gate::Or(ref id0, ref id1) => try!(env.get(id0)) | try!(env.get(id1)),
            Gate::LShift(ref id, Shift(sh)) => try!(env.get(id)) << sh,
            Gate::RShift(ref id, Shift(sh)) => try!(env.get(id)) >> sh,
        })
    }
}
impl<Ident, AltId> ExprMap<AltId> for Gate<Ident> {
    type AltExpr = Gate<AltId>;
    fn idmap<F>(&self, mut f: F) -> Gate<AltId>
        where F: FnMut(&Ident) -> AltId {
        match *self {
            Gate::Imm(val) => Gate::Imm(val),
            Gate::Not(ref id) => Gate::Not(f(id)),
            Gate::And(ref id0, ref id1) => Gate::And(f(id0), f(id1)),
            Gate::Or(ref id0, ref id1) => Gate::Or(f(id0), f(id1)),
            Gate::LShift(ref id, sh) => Gate::LShift(f(id), sh),
            Gate::RShift(ref id, sh) => Gate::RShift(f(id), sh),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Shift(u8);
impl Shift {
    fn new(sh: u8) -> Option<Shift> { if sh < 16 { Some(Shift(sh)) } else { None } }
    fn get(&self) -> u8 { self.0 }
}
