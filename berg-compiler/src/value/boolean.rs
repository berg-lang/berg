use crate::syntax::identifiers::*;
use crate::value::implement::*;

impl<'a> TryFromBergVal<'a> for bool {
    const TYPE_NAME: &'static str = "bool";
    fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, ErrorVal<'a>> {
        match from.into_val()? {
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


// Implementations for common types
impl<'a> BergValue<'a> for bool {
    fn into_val(self) -> BergResult<'a> {
        self.ok()
    }

    fn eval_val(self) -> EvalResult<'a> {
        self.ok()
    }

    fn evaluate(self) -> BergResult<'a> {
        self.into_val()
    }

    fn at_position(self, _new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.ok()
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        default_try_into_native(self)
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
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

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match operator {
            EXCLAMATION_POINT => (!self).ok(),
            DOUBLE_EXCLAMATION_POINT => self.ok(),
            _ => default_prefix(self, operator),
        }
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

    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        single_next_val(self)
    }
}
