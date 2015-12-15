use std::ops::{Add,Mul,Div,Rem};
use std::str::FromStr;

pub type Num = u64;

macro_rules! lift_ops {
    { $($tr:ident::$op:ident($lhs:ty, $rhs:ty) -> $out:ident;)* } => {
        $(impl $tr<$rhs> for $lhs {
            type Output = $out;
            fn $op(self, rhs: $rhs) -> $out {
                $out((self.0).$op(rhs.0))
            }
        })*
    }
}
macro_rules! impl_mul {
    { Scalar } => { lift_ops!{
        Mul::mul(Scalar, Scalar) -> Scalar;
    }};
    { $unit:ident } => { lift_ops!{
        Mul::mul($unit, Scalar) -> $unit;
        Mul::mul(Scalar, $unit) -> $unit;
    }}
}
macro_rules! impl_not_mul {
    { $unit:ident } => { lift_ops!{
        Add::add($unit, $unit) -> $unit;
        Div::div($unit, $unit) -> Scalar;
        Rem::rem($unit, $unit) -> $unit;
    }}
}
macro_rules! lift_from_str {
    { $($unit:ident),* } => {
        $(impl FromStr for $unit {
            type Err = <Num as FromStr>::Err;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Num::from_str(s).map($unit)
            }
        })*
    }
}

macro_rules! def_units {
    { $($unit:ident),* } => {
        $(#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
          pub struct $unit(pub Num);
          impl_mul!{ $unit }
          impl_not_mul!{ $unit }
          lift_from_str!{ $unit })*
    }
}

def_units!{ Scalar, Speed, Dist, Time, Points }

lift_ops!{
    Mul::mul(Speed, Time) -> Dist;
    Mul::mul(Time, Speed) -> Dist;
}
