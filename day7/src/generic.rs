use std::borrow::{Borrow,ToOwned};
use std::fmt::Debug;
use std::collections::HashMap;
use std::hash::Hash;

// Okay, so.  I had a few ideas here:
//
// 1. Make the language syntax parametric on identifier type, so the
//    parser can hand back a Stuff<String> and it can be interned into
//    Stuff<NewtypedInt>.  Simple enough.
//
// 2. Define the associated local semantics independently of the
//    evaluation strategy, just in terms of an abstract value
//    environment that might be recursively evaluating things.
//
// 3. Make the evaluation strategy also be parameterized over the
//    language definition.
//
// 4. Oh, and have the symbol interning thing also be parametric over
//    the language.
//
// It turns out that that much abstraction starts wanting
// higher-kinded types *really* quickly.  Trying to shoehorn all this
// into a form that actually works in Rust 1.x might have turned into
// a slightly unhealthy obsession, but... it mostly works.
//
// If nothing else, I suppose it was a learning experience in using
// associated types.

pub trait Expr {
    type Ident;
    type Value;
    fn eval<Error, F>(&self, env: F) -> Result<Self::Value, Error>
        where F: Fn(&Self::Ident) -> Result<Self::Value, Error>;
}

pub trait ExprMap<AltId>: Expr {
    type AltExpr: Expr<Ident=AltId, Value=Self::Value>; // + ExprMap<Self::Ident> for reverse map?
    fn idmap<F>(&self, f: F) -> Self::AltExpr
        where F: FnMut(&Self::Ident) -> AltId;
}

pub trait Eval<'p, P: ProgramT> {
    type Error: Debug;
    fn new(prog: &'p P) -> Self;
    fn run(&self, pc: Decl) -> Result<<P::Expr as Expr>::Value, Self::Error>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Decl(usize);
impl Decl {
    pub fn get(self) -> usize { self.0 }
}

#[derive(Clone)]
pub struct Linker<Ex: ExprMap<Decl>>
    where Ex::Ident: Eq + Hash + Clone {
    defns: Vec<Option<Ex::AltExpr>>,
    pname: Vec<Ex::Ident>,
    rname: HashMap<Ex::Ident, Decl>,
}
impl<Ex: ExprMap<Decl>> Linker<Ex>
    where Ex::Ident: Eq + Hash + Clone {
    pub fn new() -> Linker<Ex> { Linker {
        defns: Vec::new(),
        pname: Vec::new(),
        rname: HashMap::new(),
    }}
    fn lookup<BIdent: ?Sized>(&self, id: &BIdent) -> Option<Decl>
        where BIdent: Eq + Hash,
              Ex::Ident: Borrow<BIdent> {
        self.rname.get::<BIdent>(id).map(|r| *r)
    }
    fn intern<BIdent: ?Sized>(&mut self, id: &BIdent) -> Decl
        where BIdent: Eq + Hash + ToOwned<Owned=Ex::Ident>,
    // You might think that <X as ToOwned>::Owned being bounded by
    // Borrow<X> would let the typechecked infer this bound, but no:
             Ex::Ident: Borrow<BIdent> {
        if let Some(already) = self.lookup(id) {
            return already;
        }
        debug_assert!(self.defns.len() == self.pname.len());
        let decl = Decl(self.defns.len());
        self.defns.push(None);
        self.pname.push(id.to_owned());
        self.rname.insert(id.to_owned(), decl);
        decl
    }
    pub fn define<BIdent: ?Sized>(&mut self, id: &BIdent, defn: Ex)
                                  -> Result<(),LinkerError<Ex::Ident>>
        where BIdent: Eq + Hash + ToOwned<Owned=Ex::Ident>,
              Ex::Ident: Borrow<BIdent> {
        let pid = self.intern(id);
        if self.defns[pid.get()].is_some() {
            return Err(LinkerError::DuplicateDefinition(id.to_owned()));
        }
        // ...why do I have to qualify this.  Sigh.
        let pdefn = defn.idmap(|did| self.intern::<Ex::Ident>(did));
        self.defns[pid.get()] = Some(pdefn);
        Ok(())
    }
    pub fn link<BIdent: ?Sized>(self, entries: &[&BIdent]) ->
        Result<Program<Ex::AltExpr, Ex::Ident>, LinkerError<Ex::Ident>>
        where BIdent: Eq + Hash + ToOwned<Owned=Ex::Ident>,
              Ex::Ident: Borrow<BIdent> {
        let mut dentries = Vec::new();
        for &entry in entries {
            match self.lookup(entry) {
                None => {
                    return Err(LinkerError::UndefinedSymbol(entry.to_owned()));
                },
                Some(dentry) => dentries.push(dentry)
            }
        }
        let mut text = Vec::new();
        for (i, defn) in self.defns.into_iter().enumerate() {
            match defn {
                None => return Err(LinkerError::UndefinedSymbol(self.pname[i].clone())),
                Some(defn) => text.push(defn)
            }
        }
        Ok(Program {
            entries: dentries.into_boxed_slice(),
            text: text.into_boxed_slice(),
            debug_info: self.pname.into_boxed_slice()
        })
    }
}

#[derive(Debug)]
pub enum LinkerError<Ident> {
    UndefinedSymbol(Ident),
    DuplicateDefinition(Ident),
} 

pub trait ProgramT {
    type OuterIdent: Clone + Debug;
    type Expr: Expr<Ident=Decl>;
    fn entries(&self) -> &[Decl];
    fn lookup(&self, id: Decl) -> &Self::Expr;
    fn debug(&self, id: Decl) -> Self::OuterIdent;
    fn len(&self) -> usize;
}
pub struct Program<Ex, OuterIdent> {
    entries: Box<[Decl]>,
    text: Box<[Ex]>,
    debug_info: Box<[OuterIdent]>,
}
impl<Ex, OuterIdent> ProgramT for Program<Ex, OuterIdent>
        where Ex: Expr<Ident=Decl>, OuterIdent: Clone + Debug {
    type Expr = Ex;
    type OuterIdent = OuterIdent;
    // Could split this into assoc types (wanted by Eval callers) and fns
    // (wanted by Eval impls) but that seems not particularly worth it.
    fn entries(&self) -> &[Decl] { self.entries.borrow() } // (except this)
    fn lookup(&self, id: Decl) -> &Ex { &self.text[id.get()] }
    fn debug(&self, id: Decl) -> OuterIdent { self.debug_info[id.get()].clone() }
    fn len(&self) -> usize { self.text.len() }
}
