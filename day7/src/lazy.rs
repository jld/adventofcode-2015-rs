use generic::{Eval,Expr,Decl,ProgramT};
use std::cell::RefCell;
use std::marker::PhantomData;

pub type Lazy<'p, P> = GenLazy<'p, P, SafeMemo<<<P as ProgramT>::Expr as Expr>::Value>>;
pub type UnsafeLazy<'p, P> = GenLazy<'p, P, UnsafeMemo<<<P as ProgramT>::Expr as Expr>::Value>>;

pub struct GenLazy<'p, P: ProgramT + 'p, M: Memo<<P::Expr as Expr>::Value>> {
    prog: &'p P,
    memos: Box<[M]>,
}
impl<'p, P: ProgramT + 'p, M: Memo<<P::Expr as Expr>::Value>> Eval<'p, P> for GenLazy<'p, P, M>
    where <P::Expr as Expr>::Value: Clone {
    type Error = LazyError<P::OuterIdent>;
    fn new(prog: &'p P) -> Self {
        let memos: Vec<_> = (0..prog.len()).map(|_i| M::new()).collect();
        GenLazy {
            prog: prog,
            memos: memos.into_boxed_slice(),
        }
    }
    fn run(&self, entry: Decl) -> Result<<P::Expr as Expr>::Value, Self::Error> {
        self.memos[entry.get()].apply(|| {
            self.prog.lookup(entry).eval(|&pc| self.run(pc))
        }, || LazyError::Cycle(self.prog.debug(entry)))
    }
}
#[derive(Debug)]
pub enum LazyError<Ident> {
    Cycle(Ident)
}

pub trait Memo<T> {
    fn new() -> Self;
    fn apply<E, F, X>(&self, f: F, x: X) -> Result<T, E>
        where F: FnOnce() -> Result<T,E>, X: FnOnce() -> E;
}

#[allow(dead_code)]
struct NullMemo<T>(PhantomData<T>);
impl<T> Memo<T> for NullMemo<T> {
    fn new() -> Self { NullMemo(PhantomData) }
    fn apply<E, F, X>(&self, f: F, _x: X) -> Result<T, E>
        where F: FnOnce() -> Result<T,E>, X: FnOnce() -> E {
        f()
    }
}

struct UnsafeMemo<T>(RefCell<Option<T>>);
impl<T: Clone> Memo<T> for UnsafeMemo<T> {
    fn new() -> Self { UnsafeMemo(RefCell::new(None)) }
    fn apply<E, F, X>(&self, f: F, _x: X) -> Result<T, E>
        where F: FnOnce() -> Result<T,E>, X: FnOnce() -> E {
        let mut ptr = self.0.borrow_mut();
        if let Some(ref memo) = *ptr {
            return Ok(memo.clone())
        };
        match f() {
            Ok(thing) => { *ptr = Some(thing.clone()); Ok(thing) },
            err => err
        }
    }
}

struct SafeMemo<T>(RefCell<SafeMemoInner<T>>);
impl<T: Clone> Memo<T> for SafeMemo<T> {
    fn new() -> Self { SafeMemo(RefCell::new(SafeMemoInner::New)) }
    fn apply<E, F, X>(&self, f: F, x: X) -> Result<T, E>
        where F: FnOnce() -> Result<T,E>, X: FnOnce() -> E {
        {
            let mut ptr = self.0.borrow_mut();
            match *ptr {
                SafeMemoInner::Done(ref thing) => return Ok(thing.clone()),
                SafeMemoInner::Running => return Err(x()),
                SafeMemoInner::New => (),
            };
            *ptr = SafeMemoInner::Running;
        }
        let res = f();
        let mut ptr = self.0.borrow_mut();
        match res {
            Ok(thing) => { *ptr = SafeMemoInner::Done(thing.clone()); Ok(thing) },
            Err(e) => { *ptr = SafeMemoInner::New; Err(e) },
        }
    }
}
enum SafeMemoInner<T> {
    New,
    Running,
    Done(T),
}
