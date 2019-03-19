use crate::eval::BlockRef;
use crate::value::implement::*;
use num::BigRational;
use std::fmt;

///
/// A concrete type that can hold any global [`BergValue`].
/// 
/// Contrast with [`EvalVal`], which holds the results of evaluations.
///
#[derive(Clone)]
pub enum BergVal<'a> {
    ///
    /// Boolean value (true or false).
    ///
    Boolean(bool),
    ///
    /// Number value (any rational).
    ///
    BigRational(BigRational),
    ///
    /// Block value.
    /// 
    /// Operations on a block generally operate on the block's result value.
    /// Properties may be retrieved either from the result value, or from the
    /// block itself.
    /// 
    /// May or may not be evaluated already.
    /// 
    BlockRef(BlockRef<'a>),
    /// [ 1, 2, 3 ]
    Tuple(Tuple<'a>),
    /// try { 1/0 } catch { $_ }
    CaughtError(ErrorVal<'a>),
}

#[derive(Debug, Clone)]
pub struct NextVal<'a> { pub head: BergVal<'a>, pub tail: BergResult<'a> }

pub type BergResult<'a> = Result<BergVal<'a>, ErrorVal<'a>>;

impl<'a> fmt::Display for NextVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.head, self.tail.display())
    }
}
impl<'a> NextVal<'a> {
    pub fn single(value: BergVal<'a>) -> NextVal<'a> {
        NextVal { head: value, tail: Ok(BergVal::empty_tuple()) }
    }
}

impl<'a> BergVal<'a> {
    pub fn empty_tuple() -> BergVal<'a> {
        BergVal::from(vec![])
    }
}

impl<'a> From<BergVal<'a>> for EvalVal<'a> {
    fn from(from: BergVal<'a>) -> Self {
        EvalVal::Value(from)
    }
}

impl<'a> From<BergVal<'a>> for EvalResult<'a> {
    fn from(from: BergVal<'a>) -> Self {
        from.ok()
    }
}

impl<'a> BergValue<'a> for BergVal<'a> {
    fn into_val(self) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.into_val(),
            BigRational(value) => value.into_val(),
            BlockRef(value) => value.into_val(),
            Tuple(value) => value.into_val(),
            CaughtError(_) => self.ok(),
        }
    }
    fn eval_val(self) -> EvalResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.eval_val(),
            BigRational(value) => value.eval_val(),
            BlockRef(value) => value.eval_val(),
            Tuple(value) => value.eval_val(),
            CaughtError(_) => self.ok(),
        }
    }
    fn evaluate(self) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.evaluate(),
            BigRational(value) => value.evaluate(),
            BlockRef(value) => value.evaluate(),
            Tuple(value) => value.evaluate(),
            CaughtError(_) => self.ok(),
        }
    }
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.at_position(new_position),
            BigRational(value) => value.at_position(new_position),
            BlockRef(value) => value.at_position(new_position),
            Tuple(value) => value.at_position(new_position),
            CaughtError(error) => CaughtError(error.reposition(new_position)).ok(),
        }
    }

    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.next_val(),
            BigRational(value) => value.next_val(),
            BlockRef(value) => value.next_val(),
            Tuple(value) => value.next_val(),
            CaughtError(_) => single_next_val(self),
        }
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.into_native(),
            BigRational(value) => value.into_native(),
            BlockRef(value) => value.into_native(),
            Tuple(value) => value.into_native(),
            CaughtError(_) => default_into_native(self),
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.try_into_native(),
            BigRational(value) => value.try_into_native(),
            BlockRef(value) => value.try_into_native(),
            Tuple(value) => value.try_into_native(),
            CaughtError(_) => default_try_into_native(self),
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.infix(operator, right),
            BigRational(value) => value.infix(operator, right),
            BlockRef(value) => value.infix(operator, right),
            Tuple(value) => value.infix(operator, right),
            CaughtError(_) => default_infix(self, operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        use BergVal::*; 
        match self {
            Boolean(value) => value.infix_assign(operator, right),
            BigRational(value) => value.infix_assign(operator, right),
            BlockRef(value) => value.infix_assign(operator, right),
            Tuple(value) => value.infix_assign(operator, right),
            CaughtError(_) => default_infix_assign(self, operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.postfix(operator),
            BigRational(value) => value.postfix(operator),
            BlockRef(value) => value.postfix(operator),
            Tuple(value) => value.postfix(operator),
            CaughtError(_) => default_postfix(self, operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.prefix(operator),
            BigRational(value) => value.prefix(operator),
            BlockRef(value) => value.prefix(operator),
            Tuple(value) => value.prefix(operator),
            CaughtError(_) => default_prefix(self, operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.subexpression_result(boundary),
            BigRational(value) => value.subexpression_result(boundary),
            BlockRef(value) => value.subexpression_result(boundary),
            Tuple(value) => value.subexpression_result(boundary),
            CaughtError(_) => default_subexpression_result(self, boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        use BergVal::*;
        match self {
            Boolean(value) => value.field(name),
            BigRational(value) => value.field(name),
            BlockRef(value) => value.field(name),
            Tuple(value) => value.field(name),
            CaughtError(error) => match name {
                crate::syntax::identifiers::ERROR_CODE => (error.code() as usize).ok(),
                _ => default_field(CaughtError(error), name)
            }
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        field_value: BergVal<'a>,
    ) -> Result<(), ErrorVal<'a>> {
        use BergVal::*;
        match self {
            Boolean(ref mut value) => value.set_field(name, field_value),
            BigRational(ref mut value) => value.set_field(name, field_value),
            BlockRef(ref mut value) => value.set_field(name, field_value),
            Tuple(ref mut value) => value.set_field(name, field_value),
            CaughtError(ref mut value) => value.set_field(name, field_value),
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
            CaughtError(ref value) => write!(f, "{}", value)?,
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
            CaughtError(ref value) => write!(f, "{}", value),
        }
    }
}

impl<'a, V: BergValue<'a>+Clone> BergValue<'a> for Result<V, ErrorVal<'a>> {
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        match self {
            Ok(v) => v.next_val(),
            Err(v) => v.next_val(),
        }
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        match self {
            Ok(v) => v.into_native(),
            Err(v) => v.into_native(),
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        match self {
            Ok(v) => v.try_into_native(),
            Err(v) => v.try_into_native(),
        }
    }

    fn into_val(self) -> BergResult<'a> {
        match self {
            Ok(v) => v.into_val(),
            Err(v) => v.into_val(),
        }
    }

    fn eval_val(self) -> EvalResult<'a> {
        match self {
            Ok(v) => v.eval_val(),
            Err(v) => v.eval_val(),
        }
    }

    fn evaluate(self) -> BergResult<'a> {
        match self {
            Ok(v) => v.evaluate(),
            Err(v) => v.evaluate(),
        }
    }

    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        match self {
            Ok(v) => v.at_position(new_position),
            Err(v) => v.at_position(new_position),
        }
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        match self {
            Ok(v) => v.infix(operator, right),
            Err(v) => v.infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        match self {
            Ok(v) => v.infix_assign(operator, right),
            Err(v) => v.infix_assign(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Ok(v) => v.postfix(operator),
            Err(v) => v.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Ok(v) => v.prefix(operator),
            Err(v) => v.prefix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        match self {
            Ok(v) => v.subexpression_result(boundary),
            Err(v) => v.subexpression_result(boundary),
        }
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        match self {
            Ok(v) => v.field(name),
            Err(v) => v.field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> where Self: Clone {
        match self {
            Ok(v) => v.set_field(name, value),
            Err(v) => v.set_field(name, value),
        }
    }
}
