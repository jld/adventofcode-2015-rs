use std::cmp::min;
use std::ops::{Add, Mul};

// I mean why not use newtypes to make *absolutely sure* I didn't
// somehow mix up length and area even though the problem is pretty
// trivial.

type Scalar = usize;

// Error E210 means I have to use a macro for this, instead of doing
// `impl<T: Scalable> ...` or whatever and using the type system.
macro_rules! linalg_impls {
    {$Thing:ident} => {
        impl Add<$Thing> for $Thing {
            type Output = $Thing;
            fn add(self, rhs: $Thing) -> $Thing {
                $Thing(self.get() + rhs.get())
            }
        }
        impl Mul<Scalar> for $Thing {
            type Output = $Thing;
            fn mul(self, rhs: Scalar) -> $Thing {
                $Thing(self.get() * rhs)
            }
        }
        impl Mul<$Thing> for Scalar {
            type Output = $Thing;
            fn mul(self, rhs: $Thing) -> $Thing {
                $Thing(self * rhs.get())
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Length(usize);
impl Length {
    fn get(&self) -> usize { self.0 }
}
linalg_impls!{ Length }

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct Area(usize);
impl Area {
    fn get(&self) -> usize { self.0 }
}
linalg_impls!{ Area }

impl Mul for Length {
    type Output = Area;
    fn mul(self, rhs: Length) -> Area {
        Area(self.get() * rhs.get())
    }
}

// Okay, now the thing:

fn wrapping(l: Length, w: Length, h: Length) -> Area {
    2*l*w + 2*w*h + 2*h*l + min(l*w, min(w*h, h*l))
}

#[cfg(test)]
mod test {
    use super::{wrapping, Length, Area};

    macro_rules! case {
        ($l:expr, $w:expr, $h:expr => $a:expr) => {
            assert_eq!(wrapping(Length($l), Length($w), Length($h)), Area($a));
        }
    }

    #[test]
    fn spec1() {
        case!(2, 3, 4 => 58);
        case!(1, 1, 10 => 43);
    }
}

