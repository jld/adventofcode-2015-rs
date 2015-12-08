use generic::{Expr,ExprMap};

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
    fn eval<Error, F>(&self, env: F) -> Result<Self::Value, Error>
        where F: Fn(&Self::Ident) -> Result<Self::Value, Error> {
        Ok(match *self {
            Gate::Imm(val) => val,
            Gate::Not(ref id) => { !try!(env(id)) },
            Gate::And(ref id0, ref id1) => try!(env(id0)) & try!(env(id1)),
            Gate::Or(ref id0, ref id1) => try!(env(id0)) | try!(env(id1)),
            Gate::LShift(ref id, Shift(sh)) => try!(env(id)) << sh,
            Gate::RShift(ref id, Shift(sh)) => try!(env(id)) >> sh,
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
