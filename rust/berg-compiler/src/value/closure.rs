use error::{BergError,BergResult,EvalResult,Raw};
use eval::{Expression,Operand,ScopeRef};
use std::fmt;
use syntax::{AstRef,IdentifierIndex,Token};
use util::try_from::TryFrom;
use value::{BergVal,BergValue};

#[derive(Debug, Clone)]
pub struct Closure<'a>(pub Expression, pub ScopeRef<'a>);

impl<'a> fmt::Display for Closure<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.0.to_string(&self.1.ast()))
    }
}

impl<'a> BergValue<'a> for Closure<'a> {
    fn evaluate(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        self.evaluate_local()?.evaluate(scope)
    }

    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        use syntax::identifiers::*;

        if operator == DOT {
            let identifier = right.evaluate_to::<IdentifierIndex>(scope, ast)?;
            self.public_field_by_name(identifier)
        } else {
            self.evaluate_local()?.infix(operator, scope, right, ast)
        }
    }

    fn prefix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
    ) -> EvalResult<'a> {
        // Closures report their own internal error instead of local ones.
        self.evaluate_local()?.prefix(operator, scope)
    }

    fn postfix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
    ) -> EvalResult<'a> {
        self.evaluate_local()?.prefix(operator, scope)
    }
}

impl<'a> Closure<'a> {
    fn evaluate_to_object(mut self) -> BergResult<'a, (ScopeRef<'a>, BergVal<'a>, AstRef<'a>)> {
        let Closure(expression, ref mut scope) = self;
        let ast = scope.ast();
        let mut child_block = match *expression.token(&ast) {
            Token::OpenBlock { index, .. } => scope.create_child_block(index),
            _ => unreachable!(),
        };
        let result = expression.inner_expression(&ast).evaluate_local(&mut child_block, &ast)?;
        Ok((child_block, result, ast))
    }

    fn evaluate_local(self) -> BergResult<'a> {
        Ok(self.evaluate_to_object()?.1)
    }

    fn public_field_by_name(self, identifier: IdentifierIndex) -> EvalResult<'a> {
        let (child_block, result, ast) = self.evaluate_to_object()?;
        let field_result = child_block.public_field_by_name(identifier, &ast);
        if let Err(Raw(BergError::NoSuchField(..))) = field_result {
            if let BergVal::Closure(closure) = result {
                return closure.public_field_by_name(identifier);
            }
        }
        field_result
    }
}

impl<'a> From<Closure<'a>> for BergVal<'a> {
    fn from(from: Closure<'a>) -> Self {
        BergVal::Closure(from)
    }
}

impl<'a> TryFrom<BergVal<'a>> for Closure<'a> {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::Closure(closure) => Ok(closure),
            _ => Err(from),
        }
    }
}
