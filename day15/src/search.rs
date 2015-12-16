use util::best;
use ::{Stats,Num,Qty};

type Best = best::Best<Num, Vec<Qty>, best::Largest>;

fn eval(stats: &[Stats], qtys: &[Qty]) -> Num {
    debug_assert_eq!(stats.len(), qtys.len());
    stats.iter().zip(qtys.iter()).fold(Stats::zero(), |a, (s, q)| a + s.clone()**q).eval()
}

fn exh_recur(stats: &[Stats], qtys: &mut[Qty], i: usize, left: Qty, be: &mut Best) {
    if i == 0 {
        qtys[i] = left;
        be.add(eval(stats, qtys), qtys);
    } else {
        for this in 0..(left+1) {
            qtys[i] = this;
            exh_recur(stats, qtys, i - 1, left - this, be);
        }
    }
}

pub fn exhaustive(stats: &[Stats], total: Qty) -> (Num, Vec<Qty>) {
    assert!(stats.len() >= 1);
    let mut qtys = vec![!0; stats.len()];
    let mut be = Best::new(best::Largest);
    let last = qtys.len() - 1;
    exh_recur(stats, &mut qtys, last, total, &mut be);
    be.unwrap()
}

// Conjecture: this problem is amenable to hill-climbing.

#[cfg(test)]
mod tests {
    use super::exhaustive;
    use ::{Stats};

    #[test]
    fn example() {
        let bt = Stats { capacity: -1, durability: -2, flavor: 6, texture: 3, calories: 8 };
        let cn = Stats { capacity: 2, durability: 3, flavor: -2, texture: -1, calories: 3 };
        assert_eq!(exhaustive(&[bt, cn], 100), (62842880, vec![44, 56]));
    }
}
