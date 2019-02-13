use crate::eval::AmbiguousSyntax;
use crate::syntax::identifiers::*;
use crate::value::implement::*;

impl<'a> TryFromBergVal<'a> for IdentifierIndex {
    const TYPE_NAME: &'static str = "identifier";
    fn try_from_berg_val(from: BergResult<'a>) -> BergResult<'a, Result<Self, BergVal<'a>>> {
        match from.into_result() {
            Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::RawIdentifier(identifier))) => Ok(Ok(identifier)),
            Err(error) => Err(error),
            Ok(value) => Ok(Err(value)),
        }
    }
}

// Implementations for common types
impl<'a> BergValue<'a> for IdentifierIndex {
    fn into_result(self) -> BergResult<'a> {
        ControlVal::AmbiguousSyntax(AmbiguousSyntax::RawIdentifier(self)).result()
    }

    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        default_into_native(self)
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        default_try_into_native(self)
    }

    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>> {
        single_next_val(self)
    }

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a> {
        match operator {
            EQUAL_TO => match right.try_into_native::<IdentifierIndex>()? {
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
        default_prefix(self, operator)
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
}
