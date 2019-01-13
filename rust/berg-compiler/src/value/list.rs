use error::{BergResult, EvalResult};
use eval::{Operand, ScopeRef};
use syntax::{AstRef, IdentifierIndex};
use util::type_name::TypeName;
use value::*;

pub struct List<'a>(Vec<BergVal<'a>>);

impl<'a> List<'a> {
    fn concatenate<T: BergValue<'a>>(left: T, right: BergVal<'a>) -> BergResult<List<'a>> {
        let vec = Vec::new();

    }
}

impl<'a> TypeName for List<'a> {
    const TYPE_NAME: &'static str = "List";
}

impl<'a> BergValue<'a> for List<'a> {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        use syntax::identifiers::EQUAL_TO;
        match operator {
            EQUAL_TO => {
                let right = right.evaluate(scope, ast)?;
                 {
                Ok(ref value) if self.0 == *value.0 => true.ok(),
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
    fn evaluate(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        default_evaluate(self, scope)
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        default_set_field(self, name, value)
    }
}

impl<'a> fmt::Display for List<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "List<'a>")
    }
}

impl<'a> From<List<'a>> for BergVal<'a> {
    fn from(_value: List<'a>) -> Self {
        BergVal::List<'a>
    }
}

impl<'a> TryFrom<BergVal<'a>> for List<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::List<'a> => Ok(List<'a>),
            _ => Err(from),
        }
    }
}
