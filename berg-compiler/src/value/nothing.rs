use crate::syntax::IdentifierIndex;
use crate::util::try_from::TryFrom;
use crate::value::*;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Nothing;

impl TypeName for Nothing {
    const TYPE_NAME: &'static str = "nothing";
}

impl<'a> BergValue<'a> for Nothing {
    fn infix<T: BergValue<'a>>(self, operator: IdentifierIndex, _right: T) -> EvalResult<'a> {
        panic!("infix({}) called on nothing! Nothing values should never appear in Berg itself--they are sentinels for native code.", operator);
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        panic!("postfix({}) called on nothing! Nothing values should never appear in Berg itself--they are sentinels for native code.", operator);
    }
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        panic!("prefix({}) called on nothing! Nothing values should never appear in Berg itself--they are sentinels for native code.", operator);
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        panic!("field({}) called on nothing! Nothing values should never appear in Berg itself--they are sentinels for native code.", name);
    }
    fn set_field(&mut self, name: IdentifierIndex, _value: BergResult<'a>) -> EvalResult<'a, ()> {
        panic!("set_field({}) called on nothing! Nothing values should never appear in Berg itself--they are sentinels for native code.", name);
    }

    fn into_val(self) -> BergResult<'a> {
        Ok(self.into())
    }
    fn next_val(self) -> BergResult<'a, NextVal<'a>> {
        Ok(NextVal::none())
    }
}

impl fmt::Display for Nothing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "nothing")
    }
}

impl<'a> From<Nothing> for BergVal<'a> {
    fn from(_value: Nothing) -> Self {
        BergVal::Nothing
    }
}

impl<'a> TryFrom<BergVal<'a>> for Nothing {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Nothing => Ok(Nothing),
            _ => Err(from),
        }
    }
}
