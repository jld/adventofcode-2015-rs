use util::SubsetSumIter;
use ::Vol;

pub fn eggnog_iter(vols: &[Vol], target: Vol) -> Box<Iterator<Item=Vec<Vol>>> {
    Box::new(SubsetSumIter::new(vols, target).map(|(used, _unused)| used))
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
