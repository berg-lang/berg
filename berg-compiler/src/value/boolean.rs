use crate::syntax::identifiers::*;
use crate::value::implement::*;

impl<'a> TryFromBergVal<'a> for bool {
    const TYPE_NAME: &'static str = "bool";
    fn try_from_berg_val(from: BergResult<'a>) -> BergResult<'a, Result<Self, BergVal<'a>>> {
        match from.into_result()? {
            BergVal::Boolean(value) => Ok(Ok(value)),
            value => Ok(Err(value)),
        }
    }
}

impl<'a> From<bool> for BergVal<'a> {
    fn from(from: bool) -> Self {
        BergVal::Boolean(from)
    }
}

// Implementations for common types
impl<'a> BergValue<'a> for bool {
    fn into_result(self) -> BergResult<'a> {
        BergVal::Boolean(self).ok()
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        default_try_into_native(self)
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        match operator {
            AND_AND => (self && right.into_native()?).into_result(),
            OR_OR => (self || right.into_native()?).into_result(),
            EQUAL_TO => match right.try_into_native::<bool>()? {
                Some(right) => (self == right).into_result(),
                None => false.ok(),
            }
            _ => default_infix(self, operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a> {
        match operator {
            EXCLAMATION_POINT => (!self).ok(),
            DOUBLE_EXCLAMATION_POINT => self.ok(),
            _ => default_prefix(self, operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a> {
        default_postfix(self, operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a> {
        default_subexpression_result(self, boundary)
    }

    fn into_right_operand(self) -> BergResult<'a> {
        default_into_right_operand(self)
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> {
        default_set_field(self, name, value)
    }

    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        single_next_val(self)
    }
}
