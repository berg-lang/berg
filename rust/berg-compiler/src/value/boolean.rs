use error::EvalResult;
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
    ) -> EvalResult<'a> {
        match operator {
            AND_AND => (self && right.execute_to(scope, ast)?).ok(),
            OR_OR => (self || right.execute_to(scope, ast)?).ok(),
            EQUAL_TO => match right.execute(scope, ast)?.downcast::<bool>() {
                Ok(value) if self == value => true.ok(),
                _ => false.ok(),
            },
            _ => default_infix(self, operator, scope, right, ast),
        }
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match operator {
            EXCLAMATION_POINT => (!self).ok(),
            DOUBLE_EXCLAMATION_POINT => self.ok(),
            _ => default_prefix(self, operator, scope),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_postfix(self, operator, scope)
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
