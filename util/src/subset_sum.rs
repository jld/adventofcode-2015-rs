use std::iter;
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

pub struct SubsetSumIter<It: Clone + 'static>(BoxIter<Vec<It>>);
type BoxIter<T> = Box<Iterator<Item=T>>;

impl<It: Clone + 'static> SubsetSumIter<It> {
    pub fn new<Tl>(items: &[It], target: Tl) -> Self
        where Tl: Total<It> + Clone + 'static {
        SubsetSumIter(subsets_with_sum(items, target))
    }
}

impl<It: Clone + 'static> Iterator for SubsetSumIter<It> {
    type Item = Vec<It>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn subsets_with_sum<It, Tl>(vols: &[It], target: Tl) -> BoxIter<Vec<It>>
    where Tl: Total<It> + Clone + 'static, It: Clone + 'static
{
    match vols.split_last() {
        None => {
            if target.is_zero() {
                Box::new(iter::once(vec![]))
            } else {
                Box::new(iter::empty())
            }
        },
        Some((vol0, vols)) => {
            let vol0 = vol0.to_owned();
            let vols = vols.to_owned();
            Box::new(LazyIter::new(move || {
                let without = subsets_with_sum(&vols, target.clone());
                match target.checked_sub(vol0.clone()) {
                    None => without,
                    Some(ntarg) => Box::new(
                        without.chain(subsets_with_sum(&vols, ntarg)
                                      .map(move |mut acc| {
                                          acc.push(vol0.clone());
                                          acc
                                      }))) as BoxIter<Vec<It>>
                }
            }))
        }
    }
}

// FIXME: needs some unit tests.  day17 has one.

