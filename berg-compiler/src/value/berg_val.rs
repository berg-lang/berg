use crate::eval::BlockRef;
use crate::value::implement::*;
use berg_parser::ExpressionPosition;
use num::BigRational;
use std::fmt;

///
/// A concrete type that can hold any global [`BergValue`].
///
/// Contrast with [`EvalVal`], which holds the results of evaluations.
///
#[derive(Clone)]
pub enum BergVal {
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
    BlockRef(BlockRef),
    /// try { 1/0 } catch { $_ }
    CaughtException(CaughtException),
    /// compiler error
    CompilerError(CompilerError),
    /// [ 1, 2, 3 ]
    Tuple(Tuple),
}

pub type BergResult = Result<BergVal, Exception>;

impl BergVal {
    pub fn throw<T>(self) -> Result<T, EvalException> {
        EvalException::Thrown(self, ExpressionPosition::Expression).err()
    }
    pub fn is_single_primitive(&self) -> bool {
        use BergVal::*;
        match self {
            Boolean(_) | BigRational(_) | CaughtException(_) | CompilerError(_) => true,
            BlockRef(_) | Tuple(_) => false,
        }
    }
}

pub fn empty_tuple() -> BergVal {
    BergVal::from(vec![])
}

impl From<BergVal> for EvalVal {
    fn from(from: BergVal) -> Self {
        EvalVal::Val(from)
    }
}

impl From<BergVal> for EvalResult {
    fn from(from: BergVal) -> Self {
        from.ok()
    }
}

impl Value for BergVal {
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
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
    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
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

    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
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

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
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

    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl IteratorValue for BergVal {
    fn next_val(self) -> Result<NextVal, EvalException> {
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

impl ObjectValue for BergVal {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
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
        field_value: BergVal,
    ) -> Result<(), EvalException> {
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

impl OperableValue for BergVal {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
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

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
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

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
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

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
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

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
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

impl fmt::Debug for BergVal {
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

impl fmt::Display for BergVal {
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

impl BergValue for BergVal {}

impl EvaluatableValue for BergVal {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
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

impl<V: BergValue + Clone + 'static, E: BergValue + Clone + 'static> BergValue for Result<V, E> {}

impl<V: EvaluatableValue + Clone, E: EvaluatableValue + Clone> EvaluatableValue for Result<V, E> {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.evaluate(),
            Err(v) => v.evaluate(),
        }
    }
}

impl<V: Value, E: Value> Value for Result<V, E> {
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        match self {
            Ok(v) => v.into_native(),
            Err(v) => v.into_native(),
        }
    }

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        match self {
            Ok(v) => v.try_into_native(),
            Err(v) => v.try_into_native(),
        }
    }

    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.lazy_val(),
            Err(v) => v.lazy_val(),
        }
    }

    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.eval_val(),
            Err(v) => v.eval_val(),
        }
    }

    fn display(&self) -> &dyn fmt::Display {
        match self {
            Ok(v) => v.display(),
            Err(v) => v.display(),
        }
    }
}

impl<V: IteratorValue, E: IteratorValue> IteratorValue for Result<V, E> {
    fn next_val(self) -> Result<NextVal, EvalException> {
        match self {
            Ok(v) => v.next_val(),
            Err(v) => v.next_val(),
        }
    }
}

impl<V: ObjectValue + Clone, E: ObjectValue + Clone> ObjectValue for Result<V, E> {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.field(name),
            Err(v) => v.field(name),
        }
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal) -> Result<(), EvalException>
    where
        Self: Clone,
    {
        match self {
            Ok(v) => v.set_field(name, value),
            Err(v) => v.set_field(name, value),
        }
    }
}

impl<V: OperableValue, E: OperableValue> OperableValue for Result<V, E> {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.infix(operator, right),
            Err(v) => v.infix(operator, right),
        }
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.infix_assign(operator, right),
            Err(v) => v.infix_assign(operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.postfix(operator),
            Err(v) => v.postfix(operator),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.prefix(operator),
            Err(v) => v.prefix(operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        match self {
            Ok(v) => v.subexpression_result(boundary),
            Err(v) => v.subexpression_result(boundary),
        }
    }
}
