use std::iter;
use util::LazyIter;
use ::Vol;

pub type BoxIter<I> = Box<Iterator<Item=I>>;


pub fn eggnog_iter(vols: &[Vol], target: Vol) -> BoxIter<Vec<Vol>> {
    match vols.split_last() {
        None => {
            if target == 0 {
                Box::new(iter::once(vec![]))
            } else {
                Box::new(iter::empty())
            }
        },
        Some((&vol0, vols)) => {
            let vols = vols.to_owned();
            Box::new(LazyIter::new(move || {
                let without = eggnog_iter(&vols, target);
                match target.checked_sub(vol0) {
                    None => without,
                    Some(ntarg) => Box::new(without.chain(
                        eggnog_iter(&vols, ntarg)
                            .map(move |mut nog| { nog.push(vol0); nog }))) as BoxIter<_>
                }
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::eggnog_iter;

    #[test]
    fn example() {
        let vs: Vec<_> = eggnog_iter(&[20, 15, 10, 5, 5], 25).collect();
        // This winds up in the same order as the example.  What a
        // remarkable coincidence.
        assert_eq!(vs, vec![vec![15, 10],
                            vec![20, 5],
                            vec![20, 5],
                            vec![15, 5, 5]]);
    }
}
