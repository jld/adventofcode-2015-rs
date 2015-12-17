use ::{Vol, Num};

fn exhaustive(ns: &[Vol], target: Vol) -> Num {
    if ns.is_empty() {
        if target == 0 { 1 } else { 0 }
    } else {
        exhaustive(&ns[1..], target) +
            target.checked_sub(ns[0]).map_or(0, |nt| exhaustive(&ns[1..], nt))
    }
}

#[cfg(test)]
mod tests {
    use super::exhaustive;

    #[test]
    fn exh_example() {
        assert_eq!(exhaustive(&[20, 15, 10, 5, 5], 25), 4);
    }
}
