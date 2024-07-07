use crate::value::implement::*;
use berg_parser::identifiers::*;

impl BergValue for bool {}

impl EvaluatableValue for bool {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.ok()
    }
}

// Implementations for common types
impl Value for bool {
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.ok()
    }

    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        self.ok()
    }

    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        default_try_into_native(self)
    }

    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl IteratorValue for bool {
    fn next_val(self) -> Result<NextVal, EvalException> {
        single_next_val(self)
    }
}

impl ObjectValue for bool {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_field(self, name)
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal) -> Result<(), EvalException> {
        default_set_field(self, name, value)
    }
}

impl OperableValue for bool {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        match operator {
            AND_AND => (self && right.into_native()?).ok(),
            OR_OR => (self || right.into_native()?).ok(),
            EQUAL_TO => match right.try_into_native::<bool>()? {
                Some(right) => self == right,
                None => false,
            }
            .ok(),
            _ => default_infix(self, operator, right),
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
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match operator {
            EXCLAMATION_POINT => (!self).ok(),
            DOUBLE_EXCLAMATION_POINT => self.ok(),
            _ => default_prefix(self, operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_postfix(self, operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}

impl TryFromBergVal for bool {
    const TYPE_NAME: &'static str = "bool";
    fn try_from_berg_val(from: EvalVal) -> Result<Result<Self, BergVal>, EvalException> {
        match from.lazy_val()? {
            BergVal::Boolean(value) => Ok(Ok(value)),
            from => Ok(Err(from)),
        }
    }
}

impl From<bool> for BergVal {
    fn from(from: bool) -> Self {
        BergVal::Boolean(from)
    }
}

impl From<bool> for EvalVal {
    fn from(from: bool) -> Self {
        BergVal::from(from).into()
    }
}
