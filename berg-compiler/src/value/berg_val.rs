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
    /// try { 1/0 } catch { $_ }
    CaughtException(CaughtException<'a>),
    /// compiler error
    CompilerError(CompilerError<'a>),
    /// [ 1, 2, 3 ]
    Tuple(Tuple<'a>),
}

pub type BergResult<'a> = Result<BergVal<'a>, Exception<'a>>;

impl<'a> BergVal<'a> {
    pub fn throw<T>(self) -> Result<T, EvalException<'a>> {
        EvalException::Thrown(self, ExpressionErrorPosition::Expression).err()
    }
}

pub fn empty_tuple<'a>() -> BergVal<'a> {
    BergVal::from(vec![])
}

impl<'a> From<BergVal<'a>> for EvalVal<'a> {
    fn from(from: BergVal<'a>) -> Self {
        EvalVal::Val(from)
    }
}

impl<'a> From<BergVal<'a>> for EvalResult<'a> {
    fn from(from: BergVal<'a>) -> Self {
        from.ok()
    }
}

impl<'a> Value<'a> for BergVal<'a> {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.lazy_val(),
            BigRational(value) => value.lazy_val(),
            BlockRef(value) => value.lazy_val(),
            CaughtException(value) => value.lazy_val(),
            CompilerError(value) => value.lazy_val(),
            Tuple(value) => value.lazy_val(),
        }
    }
    fn eval_val(self) -> EvalResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.eval_val(),
            BigRational(value) => value.eval_val(),
            BlockRef(value) => value.eval_val(),
            CaughtException(value) => value.eval_val(),
            CompilerError(value) => value.eval_val(),
            Tuple(value) => value.eval_val(),
        }
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.into_native(),
            BigRational(value) => value.into_native(),
            BlockRef(value) => value.into_native(),
            CaughtException(value) => value.into_native(),
            CompilerError(value) => value.into_native(),
            Tuple(value) => value.into_native(),
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.try_into_native(),
            BigRational(value) => value.try_into_native(),
            BlockRef(value) => value.try_into_native(),
            CaughtException(value) => value.try_into_native(),
            CompilerError(value) => value.try_into_native(),
            Tuple(value) => value.try_into_native(),
        }
    }

    fn display(&self) -> &std::fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for BergVal<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        use BergVal::*;
        match self {
            Boolean(value) => value.next_val(),
            BigRational(value) => value.next_val(),
            BlockRef(value) => value.next_val(),
            CaughtException(value) => value.next_val(),
            CompilerError(value) => value.next_val(),
            Tuple(value) => value.next_val(),
        }
    }
}

impl<'a> ObjectValue<'a> for BergVal<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.field(name),
            BigRational(value) => value.field(name),
            BlockRef(value) => value.field(name),
            CaughtException(value) => value.field(name),
            CompilerError(value) => value.field(name),
            Tuple(value) => value.field(name),
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        field_value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>> {
        use BergVal::*;
        match self {
            Boolean(ref mut value) => value.set_field(name, field_value),
            BigRational(ref mut value) => value.set_field(name, field_value),
            BlockRef(ref mut value) => value.set_field(name, field_value),
            CaughtException(ref mut value) => value.set_field(name, field_value),
            CompilerError(ref mut value) => value.set_field(name, field_value),
            Tuple(ref mut value) => value.set_field(name, field_value),
        }
    }
}

impl<'a> OperableValue<'a> for BergVal<'a> {
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.infix(operator, right),
            BigRational(value) => value.infix(operator, right),
            BlockRef(value) => value.infix(operator, right),
            CaughtException(value) => value.infix(operator, right),
            CompilerError(value) => value.infix(operator, right),
            Tuple(value) => value.infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        use BergVal::*; 
        match self {
            Boolean(value) => value.infix_assign(operator, right),
            BigRational(value) => value.infix_assign(operator, right),
            BlockRef(value) => value.infix_assign(operator, right),
            CaughtException(value) => value.infix_assign(operator, right),
            CompilerError(value) => value.infix_assign(operator, right),
            Tuple(value) => value.infix_assign(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.postfix(operator),
            BigRational(value) => value.postfix(operator),
            BlockRef(value) => value.postfix(operator),
            CaughtException(value) => value.postfix(operator),
            CompilerError(value) => value.postfix(operator),
            Tuple(value) => value.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.prefix(operator),
            BigRational(value) => value.prefix(operator),
            BlockRef(value) => value.prefix(operator),
            CaughtException(value) => value.prefix(operator),
            CompilerError(value) => value.prefix(operator),
            Tuple(value) => value.prefix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.subexpression_result(boundary),
            BigRational(value) => value.subexpression_result(boundary),
            BlockRef(value) => value.subexpression_result(boundary),
            CaughtException(value) => value.subexpression_result(boundary),
            CompilerError(value) => value.subexpression_result(boundary),
            Tuple(value) => value.subexpression_result(boundary),
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
            CaughtException(ref value) => write!(f, "{}", value)?,
            CompilerError(ref value) => write!(f, "{}", value)?,
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
            CaughtException(ref value) => write!(f, "{}", value),
            CompilerError(ref value) => write!(f, "{}", value),
            Tuple(ref value) => write!(f, "{}", value),
        }
    }
}

impl<'a> BergValue<'a> for BergVal<'a> {}

impl<'a> EvaluatableValue<'a> for BergVal<'a> {
    fn evaluate(self) -> BergResult<'a> where Self: Sized {
        use BergVal::*;
        match self {
            Boolean(value) => value.evaluate(),
            BigRational(value) => value.evaluate(),
            BlockRef(value) => value.evaluate(),
            Tuple(value) => value.evaluate(),
            CaughtException(value) => value.evaluate(),
            CompilerError(value) => value.evaluate(),
        }
    }
}

impl<'a, V: BergValue<'a>+Clone+'a, E: BergValue<'a>+Clone+'a> BergValue<'a> for Result<V, E> {}

impl<'a, V: EvaluatableValue<'a>+Clone+'a, E: EvaluatableValue<'a>+Clone+'a> EvaluatableValue<'a> for Result<V, E> {
    fn evaluate(self) -> BergResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.evaluate(),
            Err(v) => v.evaluate(),
        }
    }
}

impl<'a, V: Value<'a>+'a, E: Value<'a>+'a> Value<'a> for Result<V, E> {
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        match self {
            Ok(v) => v.into_native(),
            Err(v) => v.into_native(),
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        match self {
            Ok(v) => v.try_into_native(),
            Err(v) => v.try_into_native(),
        }
    }

    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized {
        match self {
            Ok(v) => v.lazy_val(),
            Err(v) => v.lazy_val(),
        }
    }

    fn eval_val(self) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.eval_val(),
            Err(v) => v.eval_val(),
        }
    }

    fn display(&self) -> &fmt::Display {
        match self {
            Ok(v) => v.display(),
            Err(v) => v.display(),
        }
    }
}

impl<'a, V: IteratorValue<'a>+'a, E: IteratorValue<'a>+'a> IteratorValue<'a> for Result<V, E> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        match self {
            Ok(v) => v.next_val(),
            Err(v) => v.next_val(),
        }
    }
}

impl<'a, V: ObjectValue<'a>+Clone+'a, E: ObjectValue<'a>+Clone+'a> ObjectValue<'a> for Result<V, E> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.field(name),
            Err(v) => v.field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), EvalException<'a>> where Self: Clone {
        match self {
            Ok(v) => v.set_field(name, value),
            Err(v) => v.set_field(name, value),
        }
    }
}

impl<'a, V: OperableValue<'a>+'a, E: OperableValue<'a>+'a> OperableValue<'a> for Result<V, E> {
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.infix(operator, right),
            Err(v) => v.infix(operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.infix_assign(operator, right),
            Err(v) => v.infix_assign(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.postfix(operator),
            Err(v) => v.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.prefix(operator),
            Err(v) => v.prefix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> where Self: Sized {
        match self {
            Ok(v) => v.subexpression_result(boundary),
            Err(v) => v.subexpression_result(boundary),
        }
    }
}
