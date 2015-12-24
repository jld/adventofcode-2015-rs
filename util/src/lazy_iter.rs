use std::mem;

pub struct LazyIter<I: Iterator, F: FnOnce() -> I>(State<I, F>);

enum State<I, F> where I: Iterator, F: FnOnce() -> I {
    Future(F),
    Present,
    Past(I),
    Done,
}
impl<I, F> Iterator for LazyIter<I, F> where I: Iterator, F: FnOnce() -> I {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let early = match self.0 {
            State::Done => Some(None),
            State::Past(ref mut inner) => Some(inner.next()),
            _ => None
        };
        if let Some(rv) = early {
            if rv.is_none() {
                // Drop the inner iterator.  This is important if
                // it's the first half of a `Chain`.
                self.0 = State::Done;
            }
            return rv;
        }
        match mem::replace(&mut self.0, State::Present) {
            State::Present => panic!("circular dependency in LazyIter"),
            State::Future(f) => {
                self.0 = State::Past(f());
                self.next()
            },
            _ => unreachable!()
        }
    }
}
impl<I, F> LazyIter<I, F> where I: Iterator, F: FnOnce() -> I {
    pub fn new(f: F) -> LazyIter<I, F> {
        LazyIter(State::Future(f))
    }
}

// FIXME: needs unit tests
