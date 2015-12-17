use std::cmp::min;
use std::ops::{Add,Mul};
use ::{Vol, Num};

fn min_or<T: Ord>(v1o: Option<T>, v2: T) -> T {
    match v1o {
        None => v2,
        Some(v1) => min(v1, v2)
    }
}

fn convolve<T, U, V>(us: &[T], ts: &[U], limit: Option<usize>) -> Vec<V>
    where T: Mul<U, Output=V> + Clone,
          U: Clone,
          V: Add<V, Output=V> {
    if us.is_empty() || ts.is_empty() {
        return vec![];
    }
    // This needs a little fancy footwork to not require unstable
    // features like `Zero` or `AddAssign`.  But as a bonus, it's easy
    // to cut off the result after a certain point.
    let umax = us.len() - 1;
    let tmax = ts.len() - 1;
    let outlen = min_or(limit, umax + tmax) + 1;
    (0..outlen).map(|ij| {
        let i_lo = ij - min(ij, tmax);
        let i_hi = min(ij, umax) + 1;
        debug_assert!(i_lo < i_hi);
        let mut dot = (i_lo..i_hi).map(|i| {
            let j = ij - i;
            us[i].clone() * ts[j].clone()
        });
        let dot0 = dot.next().unwrap();
        dot.fold(dot0, |va, vb| va + vb)
    }).collect()
}

pub fn subset_sums(ns: &[Vol], limit: Option<usize>) -> Vec<Num> {
    if ns.len() == 0 {
        vec![1]
    } else if ns.len() == 1 {
        let n = ns[0] as usize;
        let mut acc = vec![0; min_or(limit, n) + 1];
        acc[0] = 1;
        if acc.len() > n {
            acc[n] = 1;
        }
        acc
    } else {
        let (ls, rs) = ns.split_at(ns.len() / 2);
        let lsss = subset_sums(ls, limit);
        let rsss = subset_sums(rs, limit);
        convolve(&lsss, &rsss, limit)
    }
}

pub fn subset_sum_at(ns: &[Vol], target: Vol) -> Num {
    // The last convolve could be optimized, but I don't quite feel
    // like threading around another parameter that's useful only in
    // the root of the recursion.  (Also the input could be
    // pre-divided by its gcd but that's not too useful here.)
    subset_sums(ns, Some(target as usize))[target as usize]
}


#[cfg(test)]
mod tests {
    use super::{convolve,subset_sums,subset_sum_at};

    #[test]
    fn conv_trivial() {
        assert_eq!(convolve(&[6], &[7], None), vec!(42));
    }

    #[test]
    fn conv_vec() {
        assert_eq!(convolve(&[2], &[3, 4, 5], None), vec![6, 8, 10]);
        assert_eq!(convolve(&[3, 4, 5], &[2], None), vec![6, 8, 10]);
    }

    #[test]
    fn conv_square() {
        assert_eq!(convolve(&[2, 3], &[5, 7], None), vec![10, 15+14, 21]);
        assert_eq!(convolve(&[1, 2, 3], &[4, 5, 6], None), vec![4, 8+5, 12+10+6, 15+12, 18]);
    }

    #[test]
    fn conv_rect() {
        assert_eq!(convolve(&[2, 3], &[5, 7, 11], None), vec![10, 15+14, 21+22, 33]);
        assert_eq!(convolve(&[1, 2, 4, 8], &[16, 32], None), vec![16, 64, 128, 256, 256]);
    }

    #[test]
    fn conv_pasc() {
        let x: &[_] = &[1, 1];
        let mut a = x.to_owned();
        a = convolve(&a, x, None);
        assert_eq!(a, vec![1, 2, 1]);
        a = convolve(x, &a, None);
        assert_eq!(a, vec![1, 3, 3, 1]);
        a = convolve(&a, x, None);
        assert_eq!(a, vec![1, 4, 6, 4, 1]);
        a = convolve(x, &a, None);
        assert_eq!(a, vec![1, 5, 10, 10, 5, 1]);
        // Hopefully that explains why this file is named `pascal.rs`.
    }

    #[test]
    fn eggnog() {
        assert_eq!(subset_sum_at(&[20, 15, 10, 5, 5], 25), 4);
    }

    #[test]
    fn eggnog_alt() {
        assert_eq!(subset_sums(&[20/5, 15/5, 10/5, 5/5, 5/5], None),
                   vec![1, 2, 2, 3, 4, 4, 4, 4, 3, 2, 2, 1]);
    }
}
