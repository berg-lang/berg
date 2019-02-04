use crate::syntax::IdentifierIndex;
use crate::util::try_from::TryFrom;
use crate::value::*;

impl TypeName for IdentifierIndex {
    const TYPE_NAME: &'static str = "identifier";
}

impl<'a> BergValue<'a> for IdentifierIndex {
    fn infix<T: BergValue<'a>>(
        self,
        operator: IdentifierIndex,
        right: T,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::EQUAL_TO;
        match operator {
            EQUAL_TO => match right.into_native::<IdentifierIndex>()? {
                Ok(value) if self == value => true.ok(),
                _ => false.ok(),
            },
            _ => default_infix(self, operator, right),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        default_postfix(self, operator)
    }
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        default_prefix(self, operator)
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

impl<'a> From<IdentifierIndex> for BergVal<'a> {
    fn from(value: IdentifierIndex) -> Self {
        BergVal::Identifier(value)
    }
}

impl<'a> TryFrom<BergVal<'a>> for IdentifierIndex {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Identifier(value) => Ok(value),
            _ => Err(from),
        }
    }
}
