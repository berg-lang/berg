use syntax::identifiers::*;
use util::try_from::TryFrom;
use value::*;

impl<'a> TypeName for bool {
    const TYPE_NAME: &'static str = "bool";
}

// Implementations for common types
impl<'a> BergValue<'a> for bool {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        match operator {
            AND_AND => (self && right.evaluate_to(scope, ast)?).ok(),
            OR_OR => (self || right.evaluate_to(scope, ast)?).ok(),
            EQUAL_TO => match right.evaluate(scope, ast)?.downcast::<bool>() {
                Ok(value) if self == value => true.ok(),
                _ => false.ok(),
            },
            _ => default_infix(self, operator, scope, right, ast),
        }
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        match operator {
            EXCLAMATION_POINT => (!self).ok(),
            DOUBLE_EXCLAMATION_POINT => (!!self).ok(),
            _ => default_prefix(self, operator, scope),
        }
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
