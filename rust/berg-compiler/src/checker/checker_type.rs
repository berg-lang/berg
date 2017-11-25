use compiler::source_data::SourceIndex;
use ast::AstIndex;
use num::BigRational;
use num::bigint::BigInt;

#[derive(Debug,Clone,PartialEq,PartialOrd)]
pub enum Type {
    Rational(BigRational),
    Boolean(bool),
    Error,
    Missing,
    Undefined { reference_source: SourceIndex, reference_index: AstIndex },
    Nothing,
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
