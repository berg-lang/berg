use std::fmt;
use syntax::IdentifierIndex;
use util::try_from::TryFrom;
use value::*;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Nothing;

impl TypeName for Nothing {
    const TYPE_NAME: &'static str = "nothing";
}

impl<'a> BergValue<'a> for Nothing {
    fn infix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>, right: Operand, ast: &AstRef<'a>) -> BergResult<'a> {
        use syntax::identifiers::EQUAL_TO;
        match operator {
            EQUAL_TO => right.evaluate(scope, ast)?.downcast::<Nothing>().is_ok().ok(),
            _ => default_infix(self, operator, scope, right, ast),
        }
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
