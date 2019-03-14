use crate::syntax::identifiers::*;
use crate::value::implement::*;

impl<'a> TryFromBergVal<'a> for IdentifierIndex {
    const TYPE_NAME: &'static str = "identifier";
    fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, ErrorVal<'a>> {
        match from {
            EvalVal::RawIdentifier(value) => Ok(Ok(value)),
            from => Ok(Err(from.into_val()?))
        }
    }
}

// Implementations for common types
impl<'a> BergValue<'a> for IdentifierIndex {
    fn into_val(self) -> BergResult<'a> {
        self.eval_val().into_val()
    }

    fn eval_val(self) -> EvalResult<'a> {
        EvalVal::RawIdentifier(self).ok()
    }

    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.into_val().at_position(new_position)
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        default_try_into_native(self)
    }

    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        single_next_val(self)
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        match operator {
            EQUAL_TO => match right.try_into_native::<IdentifierIndex>()? {
                Some(right) => self == right,
                None => false,
            }.ok(),
            _ => default_infix(self, operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        default_prefix(self, operator)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        default_postfix(self, operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        default_subexpression_result(self, boundary)
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        default_set_field(self, name, value)
    }
}
