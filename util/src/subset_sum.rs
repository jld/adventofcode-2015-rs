use std::iter;
use std::rc::Rc;
use lazy_iter::LazyIter;

pub trait Total<Item = Self>: Sized {
    fn is_zero(&self) -> bool;
    fn checked_sub(self, other: Item) -> Option<Self>;
    // z.is_zero() && !Self::from(p).is_zero() -> z.checked_sub(o) == None
    // (i.e., this is for unsigned types)
}

macro_rules! impl_total { { $($int:ty),* } => {
    $(impl<T> Total<T> for $int where $int: From<T> {
        fn is_zero(&self) -> bool { *self == 0 }
        fn checked_sub(self, other: T) -> Option<Self> {
            self.checked_sub(Self::from(other))
        }
    })*
}}
impl_total!{ u8, u16, u32, u64, usize }

pub struct SubsetSumIter<It: Copy + 'static>(BoxIter<It>);
type BoxIter<It> = Box<Iterator<Item=(Vec<It>,Vec<It>)>>;

impl<It: Copy + 'static> SubsetSumIter<It> {
    // TODO, maybe: template this harder so the unused side can be dropped.
    // Extend + Default would do it?
    pub fn new<Tl>(items: &[It], target: Tl) -> Self
        where Tl: Total<It> + Copy + 'static {
        SubsetSumIter(subsets_with_sum(Rc::new(items.to_owned()), items.len(), target))
    }
}

impl<It: Copy + 'static> Iterator for SubsetSumIter<It> {
    type Item = (Vec<It>,Vec<It>);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn subsets_with_sum<It, Tl>(vols: Rc<Vec<It>>, n: usize, target: Tl) -> BoxIter<It>
    where Tl: Total<It> + Copy + 'static, It: Copy + 'static
{
    match n.checked_sub(1) {
        None => {
            if target.is_zero() {
                Box::new(iter::once((vec![], vec![])))
            } else {
                Box::new(iter::empty())
            }
        },
        Some(n) => {
            let voln = vols[n];
            let without_f = move |vols| {
                subsets_with_sum(vols, n, target)
                    .map(move |(yes, mut no)| {
                        no.push(voln);
                        (yes, no)
                    })
            };
            let with_f = move |vols, new_target| {
                subsets_with_sum(vols, n, new_target)
                    .map(move |(mut yes, no)| {
                        yes.push(voln);
                        (yes, no)
                    })
            };
            match target.checked_sub(voln) {
                None =>
                    Box::new(LazyIter::new(move || {
                        without_f(vols)
                    })) as BoxIter<It>,
                Some(new_target) =>
                    Box::new(LazyIter::new(move || {
                        let vols_c = vols.clone();
                        without_f(vols).chain(with_f(vols_c, new_target))
                    })) as BoxIter<It>,
            }
        }
    }
}

// FIXME: needs some unit tests.  day17 has one.

