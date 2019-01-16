use syntax::IdentifierIndex;
use util::try_from::TryFrom;
use value::*;

impl TypeName for IdentifierIndex {
    const TYPE_NAME: &'static str = "identifier";
}

impl<'a> BergValue<'a> for IdentifierIndex {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        use syntax::identifiers::EQUAL_TO;
        match operator {
            EQUAL_TO => match right.execute(scope, ast)?.downcast::<IdentifierIndex>() {
                Ok(identifier) if identifier == self => true.ok(),
                _ => false.ok(),
            },
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
