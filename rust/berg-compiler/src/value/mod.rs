mod berg_val;
mod boolean;
mod identifier;
mod nothing;
mod rational;

pub use value::berg_val::BergVal;
pub use value::nothing::Nothing;

use error::{BergError, BergResult, EvalResult};
use eval::{Operand, ScopeRef};
use std::fmt;
use syntax::{AstRef, Fixity, IdentifierIndex};
use util::try_from::TryFrom;
use util::type_name::TypeName;

///
/// A value that can participate in Berg expressions.
///
pub trait BergValue<'a>: Sized + Into<BergVal<'a>> + Clone + fmt::Debug {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a>;
    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a>;
    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a>;

    // Evaluation: values which need further work to resolve, like blocks, implement this.
    fn evaluate(self, scope: &mut ScopeRef<'a>) -> BergResult<'a>;

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a>;
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()>;

    // Helpers provided to make working with values a little easier.
    fn ok<E>(self) -> Result<BergVal<'a>, E>
    where
        Self: Into<BergVal<'a>>,
    {
        Ok(self.into())
    }
    fn evaluate_to<T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
    ) -> EvalResult<'a, T> {
        self.evaluate(scope)?.downcast::<T>()
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
        NOT_EQUAL_TO => left.infix(EQUAL_TO, scope, right, ast)?
            .prefix(EXCLAMATION_POINT, scope),
        DOT => {
            let identifier = right.evaluate_to::<IdentifierIndex>(scope, ast)?;
            left.field(identifier)
        }
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

pub fn default_field<'a, T: BergValue<'a>>(object: &T, name: IdentifierIndex) -> EvalResult<'a> {
    BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into()), name).err()
}

#[cfg_attr(feature = "clippy", allow(needless_pass_by_value))]
pub fn default_set_field<'a, T: BergValue<'a>>(
    object: &mut T,
    name: IdentifierIndex,
    _value: BergResult<'a>,
) -> EvalResult<'a, ()> {
    BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into()), name).err()
}

pub fn default_evaluate<'a, T: BergValue<'a>>(
    value: T,
    _scope: &mut ScopeRef<'a>,
) -> BergResult<'a> {
    value.ok()
}
