use num::BigRational;
use num::bigint::BigInt;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Type {
    Rational(BigRational),
    Error,
    Nothing,
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
