use generic::{Eval,Expr,Decl,ProgramT,ProgResult,Strategy,Erroneous};
use std::cell::RefCell;
use std::marker::PhantomData;

pub type Lazy = GenLazy<SafeM>;
pub type UnsafeLazy = GenLazy<UnsafeM>;

// Type aliases don't bring along this kind of struct constructor, apparently, so...
#[allow(non_upper_case_globals)]
pub const Lazy: Lazy = GenLazy(SafeM);
#[allow(non_upper_case_globals)]
pub const UnsafeLazy: UnsafeLazy = GenLazy(UnsafeM);

pub struct GenLazy<M>(M);
impl<P: ProgramT, M> Erroneous<P> for GenLazy<M> {
    type Error = LazyError<P::OuterIdent>;
}
impl<'p, P: ProgramT + 'p, M> Strategy<'p, P> for GenLazy<M>
    where M: MemoFlavor<<P::Expr as Expr>::Value> {
    type Eval = GenLazyEval<'p, P, M::Memo>;
    fn load(&self, prog: &'p P) -> Self::Eval {
        let memos: Vec<_> = (0..prog.len()).map(|_i| M::Memo::new()).collect();
        GenLazyEval {
            prog: prog,
            memos: memos.into_boxed_slice(),
        }
    }
}
struct GenLazyEval<'p, P: ProgramT + 'p, M>
    where M: Memo<<P::Expr as Expr>::Value> {
    prog: &'p P,
    memos: Box<[M]>,
}
impl<'p, P: ProgramT + 'p, M> Eval<'p, P, LazyError<P::OuterIdent>> for GenLazyEval<'p, P, M>
    where M: Memo<<P::Expr as Expr>::Value> {
    fn run(&self, entry: Decl) -> ProgResult<P, LazyError<P::OuterIdent>> {
        self.memos[entry.get()].apply(|| {
            self.prog.lookup(entry).eval(|&pc| self.run(pc))
        }, || LazyError::Cycle(self.prog.debug(entry)))
    }
}
#[derive(Debug)]
pub enum LazyError<Ident> {
    Cycle(Ident)
}

pub trait MemoFlavor<T> { type Memo: Memo<T>; }
type MemoApply<M, T> = <M as MemoFlavor<T>>::Memo;
type MemoApplyP<M, P> = MemoApply<M, <<P as ProgramT>::Expr as Expr>::Value>;

pub trait Memo<T> {
    fn new() -> Self;
    fn apply<E, F, X>(&self, f: F, x: X) -> Result<T, E>
        where F: FnOnce() -> Result<T,E>, X: FnOnce() -> E;
}

#[allow(dead_code)]
struct NullM;
impl<T> MemoFlavor<T> for NullM { type Memo = NullMemo<T>; }

#[allow(dead_code)]
struct NullMemo<T>(PhantomData<T>);
impl<T> Memo<T> for NullMemo<T> {
    fn new() -> Self { NullMemo(PhantomData) }
    fn apply<E, F, X>(&self, f: F, _x: X) -> Result<T, E>
        where F: FnOnce() -> Result<T,E>, X: FnOnce() -> E {
        f()
    }
}

struct UnsafeM;
impl<T: Clone> MemoFlavor<T> for UnsafeM { type Memo = UnsafeMemo<T>; }

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

struct SafeM;
impl<T: Clone> MemoFlavor<T> for SafeM { type Memo = SafeMemo<T>; }

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
