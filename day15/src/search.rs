use util::best;
use ::{Stats,Num,Qty};

type Best = best::Best<Num, Vec<Qty>, best::Largest>;

fn eval(stats: &[Stats], qtys: &[Qty]) -> Num {
    debug_assert_eq!(stats.len(), qtys.len());
    stats.iter().zip(qtys.iter()).fold(Stats::zero(), |a, (s, q)| a + s.clone()**q).eval()
}

struct ExhCtx<'s> {
    stats: &'s [Stats],
    qtys: Vec<Qty>,
    best: Best,
}

fn exh_recur(ctx: &mut ExhCtx, i: usize, left: Qty) {
    if i == 0 {
        ctx.qtys[i] = left;
        ctx.best.add(eval(ctx.stats, &ctx.qtys), &ctx.qtys);
    } else {
        for this in 0..(left+1) {
            ctx.qtys[i] = this;
            exh_recur(ctx, i - 1, left - this);
        }
    }
}

pub fn exhaustive(stats: &[Stats], total: Qty) -> (Num, Vec<Qty>) {
    assert!(stats.len() >= 1);
    let mut ctx = ExhCtx {
        stats: stats,
        qtys: vec![!0; stats.len()],
        best: Best::new(best::Largest),
    };
    exh_recur(&mut ctx, stats.len() - 1, total);
    ctx.best.unwrap()
}

// Conjecture: this problem is amenable to hill-climbing.  ...without
// the part 2 addendum, at least.  Probably still is even with it,
// with a little more subtlety, since everything except the final
// metric is linear.

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
