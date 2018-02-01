mod berg_error;
mod berg_val;
mod boolean;
mod nothing;
mod rational;

pub use value::berg_error::{BergError, ErrorCode, ErrorLocation};
pub(crate) use value::berg_error::BergErrorStack;
pub use value::berg_val::BergVal;
pub use value::nothing::Nothing;

use syntax::{AstRef, Fixity, IdentifierIndex};
use eval::{Expression, Operand, ScopeRef};
use std::fmt;
use util::type_name::TypeName;

///
/// A value that can participate in Berg expressions.
///
pub trait BergValue<'a>: fmt::Debug + Into<BergVal<'a>> + Sized {
    fn complete(self) -> BergResult<'a> {
        self.ok()
    }

    #[allow(unused_variables)]
    fn unwind_error(self, ast: AstRef<'a>, expression: Expression) -> BergVal<'a> {
        self.into()
    }

    fn ok<E>(self) -> Result<BergVal<'a>, E> {
        Ok(self.into())
    }

    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> BergResult<'a> {
        default_infix(self, operator, scope, right, ast)
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        default_postfix(self, operator, scope)
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> BergResult<'a> {
        default_prefix(self, operator, scope)
    }
}

pub fn default_infix<'a, T: BergValue<'a>>(
    left: T,
    operator: IdentifierIndex,
    scope: &mut ScopeRef<'a>,
    right: Operand,
    ast: &AstRef<'a>,
) -> BergResult<'a> {
    use syntax::identifiers::{EQUAL_TO, EXCLAMATION_POINT, NEWLINE, NOT_EQUAL_TO, SEMICOLON};
    match operator {
        SEMICOLON | NEWLINE => right.evaluate(scope, ast),
        EQUAL_TO => false.ok(),
        NOT_EQUAL_TO => left.infix(EQUAL_TO, scope, right, ast)?
            .prefix(EXCLAMATION_POINT, scope),
        _ => BergError::UnsupportedOperator(Box::new(left.into()), Fixity::Infix, operator).err(),
    }
}

pub fn default_postfix<'a, T: BergValue<'a>>(
    operand: T,
    operator: IdentifierIndex,
    _scope: &mut ScopeRef<'a>,
) -> BergResult<'a> {
    BergError::UnsupportedOperator(Box::new(operand.into()), Fixity::Postfix, operator).err()
}

pub fn default_prefix<'a, T: BergValue<'a>>(
    operand: T,
    operator: IdentifierIndex,
    scope: &mut ScopeRef<'a>,
) -> BergResult<'a> {
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

pub type BergResult<'a, V = BergVal<'a>, E = BergVal<'a>> = Result<V, E>;
