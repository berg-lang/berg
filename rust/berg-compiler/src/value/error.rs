use error::Error;
use util::try_from::TryFrom;
use value::*;

impl<'a> TypeName for Box<Error<'a>> {
    const TYPE_NAME: &'static str = "Error";
}

impl<'a> BergValue<'a> for Box<Error<'a>> {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        // TODO add EQUAL_TO
        default_infix(self, operator, scope, right, ast)
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

impl<'a> From<Box<Error<'a>>> for BergVal<'a> {
    fn from(value: Box<Error<'a>>) -> Self {
        BergVal::Error(value)
    }
}

impl<'a> TryFrom<BergVal<'a>> for Box<Error<'a>> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Error(value) => Ok(value),
            _ => Err(from),
        }
    }
}

impl<'a> From<Error<'a>> for BergVal<'a> {
    fn from(value: Error<'a>) -> Self {
        BergVal::Error(Box::new(value))
    }
}

impl<'a> TryFrom<BergVal<'a>> for Error<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Error(value) => Ok(*value),
            _ => Err(from),
        }
    }
}
