use crate::syntax::identifiers::*;
use crate::value::implement::*;

impl<'a> TryFromBergVal<'a> for IdentifierIndex {
    const TYPE_NAME: &'static str = "identifier";
    fn try_from_berg_val(
        from: EvalVal<'a>,
    ) -> Result<Result<Self, BergVal<'a>>, EvalException<'a>> {
        match from {
            EvalVal::RawIdentifier(value) => Ok(Ok(value)),
            from => Ok(Err(from.lazy_val()?)),
        }
    }
}

// Implementations for common types
impl<'a> Value<'a> for IdentifierIndex {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>>
    where
        Self: Sized,
    {
        self.eval_val().lazy_val()
    }

    fn eval_val(self) -> EvalResult<'a>
    where
        Self: Sized,
    {
        EvalVal::RawIdentifier(self).ok()
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        default_try_into_native(self)
    }

    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for IdentifierIndex {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        single_next_val(self)
    }
}

impl<'a> ObjectValue<'a> for IdentifierIndex {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        default_field(self, name)
    }
    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>> {
        default_set_field(self, name, value)
    }
}

impl<'a> OperableValue<'a> for IdentifierIndex {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
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
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        default_prefix(self, operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        default_postfix(self, operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a>
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}
