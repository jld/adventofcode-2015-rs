use std::cmp::min;
use std::io::{stdin, BufRead};
use std::ops::{Add, Mul};
use std::str::FromStr;

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

// But as long as I'm using macros...
macro_rules! unit_def {
    {$Thing:ident} => {
        #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
        struct $Thing(Scalar);
        impl $Thing {
            fn get(&self) -> Scalar { self.0 }
        }
        linalg_impls!{ $Thing }
    }
}

macro_rules! unit_mul_impl {
    {$Left:ident * $Right: ident -> $Out:ident} => {
        impl Mul<$Right> for $Left {
            type Output = $Out;
            fn mul(self, rhs: $Right) -> $Out {
                $Out(self.get() * rhs.get())
            }
        }
    }
}

unit_def!{ Length }
unit_def!{ Area }
unit_def!{ Volume }
unit_mul_impl!{ Length * Length -> Area }
unit_mul_impl!{ Length * Area -> Volume }
unit_mul_impl!{ Area * Length -> Volume }

// Okay, now the thing:

fn min3<T>(v1: T, v2: T, v3: T) -> T where T: Ord {
    min(min(v1, v2), v3)
}

fn wrapping(l: Length, w: Length, h: Length) -> Area {
    2*l*w + 2*w*h + 2*h*l + min3(l*w, w*h, h*l)
}

fn elf_bow_magic(v: Volume) -> Length {
    Length(v.get())
}

fn bow(l: Length, w: Length, h: Length) -> Length {
    elf_bow_magic(l * w * h)
}

fn ribbons(l: Length, w: Length, h: Length) -> Length {
    min3(2*(l + w), 2*(w + h), 2*(h + l)) + bow(l, w, h)
}

#[derive(Debug, Clone)]
struct Stuff {
    wrapping: Area,
    ribbons: Length,
}
impl Stuff {
    fn zero() -> Stuff { Stuff { wrapping: Area(0), ribbons: Length(0) }}
    fn needed(l: Length, w: Length, h: Length) -> Stuff { Stuff {
        wrapping: wrapping(l, w, h),
        ribbons: ribbons(l, w, h)
    }}
}
impl Add for Stuff {
    type Output = Stuff;
    fn add(self, rhs: Stuff) -> Stuff { Stuff {
        wrapping: self.wrapping + rhs.wrapping,
        ribbons: self.ribbons + rhs.ribbons
    }}
}

pub fn main() {
    let stdin = stdin();
    let total: Stuff = stdin.lock().lines().map(|line| {
        let dims: Vec<_> =
            line.expect("I/O error reading stdin")
                .split('x')
                .map(|s| Length(usize::from_str(s).expect("not a number")))
                .collect();
        assert_eq!(dims.len(), 3);
        Stuff::needed(dims[0], dims[1], dims[2])
    }).fold(Stuff::zero(), |aa, a| { aa + a });
    // (If I wanted to use unstable stuff, I could throw a `Zero` impl
    // into `linalg_impls!` and just do `.sum()` instead of that last
    // thing.  But no.)
    println!("Wrapping paper: {} ft²", total.wrapping.get());
    println!("Ribbons: {} ft", total.ribbons.get());
}

#[cfg(test)]
mod test {
    use super::{wrapping, ribbons, Length, Area};

    macro_rules! wrap_case {
        ($l:expr, $w:expr, $h:expr => $a:expr) => {
            assert_eq!(wrapping(Length($l), Length($w), Length($h)), Area($a));
        }
    }
    macro_rules! rib_case {
        ($l:expr, $w:expr, $h:expr => $a:expr) => {
            assert_eq!(ribbons(Length($l), Length($w), Length($h)), Length($a));
        }
    }

    #[test]
    fn spec_wrap() {
        wrap_case!(2, 3, 4 => 58);
        wrap_case!(1, 1, 10 => 43);
    }

    #[test]
    fn spec_rib() {
        rib_case!(2, 3, 4 => 34);
        rib_case!(1, 1, 10 => 14);
    }
}

