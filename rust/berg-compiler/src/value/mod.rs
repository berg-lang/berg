mod berg_val;
mod boolean;
mod closure;
mod identifier;
mod nothing;
mod rational;

pub use value::berg_val::BergVal;
pub use value::closure::Closure;
pub use value::nothing::Nothing;

use error::{BergResult, EvalResult, BergError};
use eval::{Operand, ScopeRef};
use std::fmt;
use syntax::{AstRef, Fixity, IdentifierIndex};
use util::type_name::TypeName;

///
/// A value that can participate in Berg expressions.
///
pub trait BergValue<'a>: Sized+Into<BergVal<'a>> {
    // Handles EvalResult (which is what you normally need to convert to)
    fn ok<E>(self) -> Result<BergVal<'a>, E> where Self: Into<BergVal<'a>> {
        Ok(self.into())
    }

    fn infix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>, right: Operand, ast: &AstRef<'a>) -> EvalResult<'a> {
        default_infix(self, operator, scope, right, ast)
    }
    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_postfix(self, operator, scope)
    }
    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        default_prefix(self, operator, scope)
    }
    #[allow(unused_variables)]
    fn evaluate(self, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        self.ok()
    }
}

pub fn default_infix<'a, T: BergValue<'a>>(
    left: T,
    operator: IdentifierIndex,
    scope: &mut ScopeRef<'a>,
    right: Operand,
    ast: &AstRef<'a>,
) -> EvalResult<'a> {
    use syntax::identifiers::{DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE, NOT_EQUAL_TO, SEMICOLON};
    match operator {
        SEMICOLON | NEWLINE => Ok(right.evaluate_local(scope, ast)?),
        EQUAL_TO => false.ok(),
        NOT_EQUAL_TO => left.infix(EQUAL_TO, scope, right, ast)?.prefix(EXCLAMATION_POINT, scope),
        DOT => BergError::NoSuchPublicFieldOnValue(Box::new(left.into()), right.evaluate_to::<IdentifierIndex>(scope, ast)?).err(),
        _ => BergError::UnsupportedOperator(Box::new(left.into()), Fixity::Infix, operator).err(),
    }
}

pub fn default_postfix<'a, T: BergValue<'a>>(
    operand: T,
    operator: IdentifierIndex,
    _scope: &mut ScopeRef<'a>,
) -> EvalResult<'a> {
    BergError::UnsupportedOperator(Box::new(operand.into()), Fixity::Postfix, operator).err()
}

pub fn default_prefix<'a, T: BergValue<'a>>(
    operand: T,
    operator: IdentifierIndex,
    scope: &mut ScopeRef<'a>,
) -> EvalResult<'a> {
    use syntax::identifiers::{DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT};
    match operator {
        DOUBLE_EXCLAMATION_POINT => operand
            .prefix(EXCLAMATION_POINT, scope)?
            .prefix(EXCLAMATION_POINT, scope),
        _ => {
            BergError::UnsupportedOperator(Box::new(operand.into()), Fixity::Prefix, operator).err()
        }
    }
}
