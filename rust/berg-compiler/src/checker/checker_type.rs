use std::fmt;
use std::fmt::{Display,Formatter};
use num::BigRational;
use num::bigint::BigInt;

#[derive(Debug,Clone,PartialEq,PartialOrd)]
pub enum Type {
    Rational(BigRational),
    Boolean(bool),
    Error,
    Missing,
    Nothing,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use checker::checker_type::Type::*;
        match *self {
            Rational(ref value) => write!(f, "{}", value),
            Boolean(ref value) => write!(f, "{}", value),
            Error => write!(f, "{}", "error"),
            Missing => write!(f, "{}", "missing"),
            Nothing => write!(f, "{}", "nothing"),
       }
    }
}

impl From<bool> for Type {
    fn from(value: bool) -> Type {
        Type::Boolean(value)
    }
}
impl From<i64> for Type {
    fn from(value: i64) -> Type {
        BigInt::from(value).into()
    }
}
impl From<BigInt> for Type {
    fn from(value: BigInt) -> Type {
        BigRational::from(value).into()
    }
}
impl From<BigRational> for Type {
    fn from(value: BigRational) -> Type {
        Type::Rational(value)
    }
}
