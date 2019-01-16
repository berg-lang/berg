mod berg_val;
mod boolean;
// mod error;
mod identifier;
mod nothing;
mod rational;
mod tuple;

pub use self::berg_val::BergVal;
pub use self::nothing::Nothing;
pub use self::tuple::Tuple;

use crate::error::{BergError, BergResult, EvalResult};
use crate::eval::{Operand, ScopeRef};
use crate::syntax::{AstRef, Fixity, IdentifierIndex};
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use std::fmt;

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

    ///
    /// Executes this value if it is lazy, returning the final value.
    ///
    fn result(self, scope: &mut ScopeRef<'a>) -> BergResult<'a>;

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a>;
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()>;

    // Helpers provided to make working with values a little easier.
    fn ok<E>(self) -> Result<BergVal<'a>, E>
    where
        Self: Into<BergVal<'a>>,
    {
        Ok(self.into())
    }

    ///
    /// Convert this value to the given type, executing it if it is lazy.
    ///
    /// If the actual code fails, returns Ok(Err(error)). If the conversion
    /// fails, returns Err(error) so that the caller can take responsibility
    /// for its desire to convert the value.
    ///
    fn result_to<T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
        scope: &mut ScopeRef<'a>,
    ) -> EvalResult<'a, T> {
        self.result(scope)?.downcast::<T>()
    }
}

pub fn default_infix<'a, T: BergValue<'a>>(
    left: T,
    operator: IdentifierIndex,
    scope: &mut ScopeRef<'a>,
    right: Operand,
    ast: &AstRef<'a>,
) -> EvalResult<'a> {
    use crate::syntax::identifiers::{
        COMMA, DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE, NOT_EQUAL_TO, SEMICOLON,
    };
    match operator {
        SEMICOLON | NEWLINE => Ok(right.evaluate(scope, ast)?),
        COMMA => unreachable!(),
        EQUAL_TO => false.ok(),
        NOT_EQUAL_TO => left
            .infix(EQUAL_TO, scope, right, ast)?
            .prefix(EXCLAMATION_POINT, scope),
        DOT => {
            let identifier = right.execute_to::<IdentifierIndex>(scope, ast)?;
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
    use crate::syntax::identifiers::{DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT};
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

#[allow(clippy::needless_pass_by_value)]
pub fn default_set_field<'a, T: BergValue<'a>>(
    object: &mut T,
    name: IdentifierIndex,
    _value: BergResult<'a>,
) -> EvalResult<'a, ()> {
    BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into()), name).err()
}

pub fn default_result<'a, T: BergValue<'a>>(value: T, _scope: &mut ScopeRef<'a>) -> BergResult<'a> {
    value.ok()
}
