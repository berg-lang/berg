use crate::value::implement::*;
use berg_parser::identifiers::*;

impl TryFromBergVal for IdentifierIndex {
    const TYPE_NAME: &'static str = "identifier";
    fn try_from_berg_val(from: EvalVal) -> Result<Result<Self, BergVal>, EvalException> {
        match from {
            EvalVal::RawIdentifier(value) => Ok(Ok(value)),
            from => Ok(Err(from.lazy_val()?)),
        }
    }
}

// Implementations for common types
impl Value for IdentifierIndex {
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.eval_val().lazy_val()
    }

    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        EvalVal::RawIdentifier(self).ok()
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

impl IteratorValue for IdentifierIndex {
    fn next_val(self) -> Result<NextVal, EvalException> {
        single_next_val(self)
    }
}

impl ObjectValue for IdentifierIndex {
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

impl OperableValue for IdentifierIndex {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        match operator {
            EQUAL_TO => match right.try_into_native::<IdentifierIndex>()? {
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
        default_prefix(self, operator)
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
