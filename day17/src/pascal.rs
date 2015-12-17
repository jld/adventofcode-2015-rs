use std::cmp::min;
use std::ops::{Add,Mul};

fn convolve<T, U, V>(us: &[T], ts: &[U]) -> Vec<V>
    where T: Mul<U, Output=V> + Clone,
          U: Clone,
          V: Add<V, Output=V> {
    if us.is_empty() || ts.is_empty() {
        return vec![];
    }
    // This needs a little fancy footwork to not require unstable
    // features like `Zero` or `AddAssign`.
    (0..(us.len() + ts.len() - 1)).map(|ij| {
        let i_lo = ij.checked_sub(ts.len() - 1).unwrap_or(0);
        let i_hi = min(ij + 1, us.len());
        debug_assert!(i_lo < i_hi);
        let mut dot = (i_lo..i_hi).map(|i| {
            let j = ij - i;
            us[i].clone() * ts[j].clone()
        });
        let dot0 = dot.next().unwrap();
        dot.fold(dot0, |va, vb| va + vb)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::convolve;

    #[test]
    fn conv_trivial() {
        assert_eq!(convolve(&[6], &[7]), vec!(42));
    }

    #[test]
    fn conv_vec() {
        assert_eq!(convolve(&[2], &[3, 4, 5]), vec![6, 8, 10]);
        assert_eq!(convolve(&[3, 4, 5], &[2]), vec![6, 8, 10]);
    }

    #[test]
    fn conv_square() {
        assert_eq!(convolve(&[2, 3], &[5, 7]), vec![10, 15+14, 21]);
        assert_eq!(convolve(&[1, 2, 3], &[4, 5, 6]), vec![4, 8+5, 12+10+6, 15+12, 18]);
    }

    #[test]
    fn conv_rect() {
        assert_eq!(convolve(&[2, 3], &[5, 7, 11]), vec![10, 15+14, 21+22, 33]);
        assert_eq!(convolve(&[1, 2, 4, 8], &[16, 32]), vec![16, 64, 128, 256, 256]);
    }

    #[test]
    fn conv_pasc() {
        let x: &[_] = &[1, 1];
        let mut a = x.to_owned();
        a = convolve(&a, x);
        assert_eq!(a, vec![1, 2, 1]);
        a = convolve(x, &a);
        assert_eq!(a, vec![1, 3, 3, 1]);
        a = convolve(&a, x);
        assert_eq!(a, vec![1, 4, 6, 4, 1]);
        a = convolve(x, &a);
        assert_eq!(a, vec![1, 5, 10, 10, 5, 1]);
        // Hopefully that explains why this file is named `pascal.rs`.
    }
}
