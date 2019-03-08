use crate::eval::BlockRef;
use crate::value::*;
use num::BigRational;
use std::fmt;

///
/// A concrete type that can hold any `BergValue`, and delegates operations to the concrete type.
///
#[derive(Clone)]
pub enum BergVal<'a> {
    Boolean(bool),
    BigRational(BigRational),
    BlockRef(BlockRef<'a>),
    Tuple(Tuple<'a>),
}

#[derive(Debug, Clone)]
pub struct NextVal<'a> { pub head: BergResult<'a>, pub tail: BergResult<'a> }

impl<'a> fmt::Display for NextVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.head.display(), self.tail.display())
    }
}
impl<'a> NextVal<'a> {
    pub fn single(value: BergResult<'a>) -> NextVal<'a> {
        NextVal { head: value, tail: Ok(BergVal::empty_tuple()) }
    }
}

impl<'a> BergVal<'a> {
    pub fn empty_tuple() -> BergVal<'a> {
        BergVal::from(vec![])
    }
    pub fn ok<E>(self) -> Result<BergVal<'a>, E> {
        Ok(self)
    }
}

impl<'a> From<BergVal<'a>> for BergResult<'a> {
    fn from(from: BergVal<'a>) -> Self {
        Ok(from)
    }
}

impl<'a> BergValue<'a> for BergVal<'a> {
    fn into_val(self) -> BergResult<'a> {
        Ok(self)
    }

    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.next_val(),
            BigRational(value) => value.next_val(),
            BlockRef(value) => value.next_val(),
            Tuple(value) => value.next_val(),
        }
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        use BergVal::*;
        match self {
            Boolean(value) => value.into_native(),
            BigRational(value) => value.into_native(),
            BlockRef(value) => value.into_native(),
            Tuple(value) => value.into_native()
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.try_into_native(),
            BigRational(value) => value.try_into_native(),
            BlockRef(value) => value.try_into_native(),
            Tuple(value) => value.try_into_native()
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.infix(operator, right),
            BigRational(value) => value.infix(operator, right),
            BlockRef(value) => value.infix(operator, right),
            Tuple(value) => value.infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        use BergVal::*; 
        match self {
            Boolean(value) => value.infix_assign(operator, right),
            BigRational(value) => value.infix_assign(operator, right),
            BlockRef(value) => value.infix_assign(operator, right),
            Tuple(value) => value.infix_assign(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.postfix(operator),
            BigRational(value) => value.postfix(operator),
            BlockRef(value) => value.postfix(operator),
            Tuple(value) => value.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.prefix(operator),
            BigRational(value) => value.prefix(operator),
            BlockRef(value) => value.prefix(operator),
            Tuple(value) => value.prefix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.subexpression_result(boundary),
            BigRational(value) => value.subexpression_result(boundary),
            BlockRef(value) => value.subexpression_result(boundary),
            Tuple(value) => value.subexpression_result(boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a, BergResult<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.field(name),
            BigRational(value) => value.field(name),
            BlockRef(value) => value.field(name),
            Tuple(value) => value.field(name),
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        field_value: BergResult<'a>,
    ) -> BergResult<'a, ()> {
        use BergVal::*;
        match self {
            Boolean(ref mut value) => value.set_field(name, field_value),
            BigRational(ref mut value) => value.set_field(name, field_value),
            BlockRef(ref mut value) => value.set_field(name, field_value),
            Tuple(ref mut value) => value.set_field(name, field_value),
        }
    }
}

impl<'a> fmt::Debug for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BergVal::*;
        write!(f, "BergVal(")?;
        match self {
            Boolean(value) => write!(f, "{}", value)?,
            BigRational(ref value) => write!(f, "{}", value)?,
            BlockRef(ref value) => write!(f, "{}", value)?,
            Tuple(ref value) => write!(f, "{}", value)?,
        }
        write!(f, ")")
    }
}

impl<'a> fmt::Display for BergVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BergVal::*;
        match *self {
            Boolean(value) => write!(f, "{}", value),
            BigRational(ref value) => write!(f, "{}", value),
            BlockRef(ref value) => write!(f, "{}", value),
            Tuple(ref value) => write!(f, "{}", value),
        }
    }
}
