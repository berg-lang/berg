use crate::syntax::identifiers::*;
use crate::value::implement::*;

impl<'a> BergValue<'a> for bool {}

impl<'a> EvaluatableValue<'a> for bool {
    fn evaluate(self) -> BergResult<'a> where Self: Sized {
        self.ok()
    }
}

// Implementations for common types
impl<'a> Value<'a> for bool {
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized {
        self.ok()
    }

    fn eval_val(self) -> EvalResult<'a> where Self: Sized {
        self.ok()
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        default_try_into_native(self)
    }

    fn display(&self) -> &std::fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for bool {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        single_next_val(self)
    }
}

impl<'a> ObjectValue<'a> for bool {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        default_field(self, name)
    }

    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), EvalException<'a>> {
        default_set_field(self, name, value)
    }
}

impl<'a> OperableValue<'a> for bool {
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        match operator {
            AND_AND => (self && right.into_native()?).ok(),
            OR_OR => (self || right.into_native()?).ok(),
            EQUAL_TO => match right.try_into_native::<bool>()? {
                Some(right) => self == right,
                None => false,
            }.ok(),
            _ => default_infix(self, operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        match operator {
            EXCLAMATION_POINT => (!self).ok(),
            DOUBLE_EXCLAMATION_POINT => self.ok(),
            _ => default_prefix(self, operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized {
        default_postfix(self, operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> where Self: Sized {
        default_subexpression_result(self, boundary)
    }
}

impl<'a> TryFromBergVal<'a> for bool {
    const TYPE_NAME: &'static str = "bool";
    fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, EvalException<'a>> {
        match from.lazy_val()? {
            BergVal::Boolean(value) => Ok(Ok(value)),
            from => Ok(Err(from))
        }
    }
}

impl<'a> From<bool> for BergVal<'a> {
    fn from(from: bool) -> Self {
        BergVal::Boolean(from)
    }
}

impl<'a> From<bool> for EvalVal<'a> {
    fn from(from: bool) -> Self {
        BergVal::from(from).into()
    }
}
