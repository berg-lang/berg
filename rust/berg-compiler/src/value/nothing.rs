use std::fmt;
use crate::syntax::IdentifierIndex;
use crate::util::try_from::TryFrom;
use crate::value::*;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Nothing;

impl TypeName for Nothing {
    const TYPE_NAME: &'static str = "nothing";
}

impl<'a> BergValue<'a> for Nothing {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::EQUAL_TO;
        match operator {
            EQUAL_TO => right
                .execute(scope, ast)?
                .downcast::<Nothing>()
                .is_ok()
                .ok(),
            _ => default_infix(self, operator, scope, right, ast),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_postfix(self, operator, scope)
    }
    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_prefix(self, operator, scope)
    }

    // Evaluation: values which need further work to resolve, like blocks, implement this.
    fn result(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        default_result(self, scope)
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        default_set_field(self, name, value)
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
