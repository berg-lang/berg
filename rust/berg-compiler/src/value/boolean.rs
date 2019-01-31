use crate::syntax::identifiers::*;
use crate::util::try_from::TryFrom;
use crate::value::*;

impl<'a> TypeName for bool {
    const TYPE_NAME: &'static str = "bool";
}

// Implementations for common types
impl<'a> BergValue<'a> for bool {
    fn infix<T: BergValue<'a>>(
        self,
        operator: IdentifierIndex,
        right: T,
    ) -> EvalResult<'a> {
        match operator {
            AND_AND => (self && right.into_native()??).ok(),
            OR_OR => (self || right.into_native()??).ok(),
            EQUAL_TO => match right.into_native::<bool>()? { Ok(right) => self == right, Err(_) => false }.ok(),
            _ => default_infix(self, operator, right),
        }
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

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        default_set_field(self, name, value)
    }

    fn into_val(self) -> BergResult<'a> {
        Ok(self.into())
    }
    fn next_val(self) -> BergResult<'a, NextVal<'a>> {
        Ok(NextVal::single(self.into()))
    }
}

impl<'a> From<bool> for BergVal<'a> {
    fn from(value: bool) -> Self {
        BergVal::Boolean(value)
    }
}

impl<'a> TryFrom<BergVal<'a>> for bool {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Boolean(value) => Ok(value),
            _ => Err(from),
        }
    }
}
