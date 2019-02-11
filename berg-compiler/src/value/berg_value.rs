use crate::value::*;
use crate::syntax::Fixity;
use std::fmt;

///
/// A value that can participate in Berg expressions.
///
pub trait BergValue<'a>: Sized + fmt::Debug {
    fn infix<T: BergValue<'a>>(self, operator: IdentifierIndex, right: T) -> BergResult<'a>;
    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a>;
    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a>;

    fn field(&self, name: IdentifierIndex) -> BergResult<'a>;
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()>;

    // Helpers provided to make working with values a little easier.
    fn ok<T, E>(self) -> Result<T, E>
    where
        Self: Into<T>,
    {
        Ok(self.into())
    }

    fn next_val(self) -> BergResult<'a, NextVal<'a>>;

    ///
    /// Convert this to a BergVal.
    ///
    /// This largely exists because expressions must be evaluated immediately
    /// and cannot be treated as lazy; therefore, ExpressionEvaluator must
    /// completely evaluate when it converts to a val.
    ///
    fn into_val(self) -> BergResult<'a>;

    ///
    /// Get the result of this value as a particular native type.
    /// If it's a *block,* it evaluates the value and tries to convert it.
    ///
    fn into_native<T: TypeName + TryFrom<BergVal<'a>>>(self) -> BergResult<'a, BergResult<'a, T>>
    where
        <T as TryFrom<BergVal<'a>>>::Error: Into<BergVal<'a>>,
    {
        // NOTE: this means BergVal must implement its own into_native()
        self.into_val()?.into_native()
    }
}

pub fn default_infix<'a, L: BergValue<'a> + Into<BergVal<'a>>, R: BergValue<'a>>(
    left: L,
    operator: IdentifierIndex,
    right: R,
) -> BergResult<'a> {
    use crate::syntax::identifiers::{
        COMMA, DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE, NOT_EQUAL_TO, SEMICOLON,
    };
    match operator {
        SEMICOLON | NEWLINE => right.into_val()?.ok(),
        COMMA => unreachable!(),
        EQUAL_TO => {
            let mut left_next = left.next_val()?;
            let mut right_next = right.next_val()?;
            loop {
                match (left_next, right_next) {
                    (NextVal(left), NextVal(right)) => match (left, right) {
                        (None, None) => return true.ok(),
                        (Some(..), None) | (None, Some(..)) => return false.ok(),
                        (Some((left, left_tail)), Some((right, right_tail))) => {
                            // If they are the same, grab the next values and loop.
                            if left.infix(EQUAL_TO, right)?.into_native::<bool>()?? {
                                left_next = if let Some(left_tail) = left_tail {
                                    left_tail.next_val()?
                                } else {
                                    NextVal::none()
                                };
                                right_next = if let Some(right_tail) = right_tail {
                                    right_tail.next_val()?
                                } else {
                                    NextVal::none()
                                };
                            } else {
                                return false.ok();
                            }
                        }
                    },
                }
            }
        }
        NOT_EQUAL_TO => left.infix(EQUAL_TO, right)?.prefix(EXCLAMATION_POINT),
        DOT => left.field(right.into_native::<IdentifierIndex>()??),
        _ => BergError::UnsupportedOperator(Box::new(left.into()), Fixity::Infix, operator).err(),
    }
}

pub fn default_postfix<'a, T: BergValue<'a> + Into<BergVal<'a>>>(
    operand: T,
    operator: IdentifierIndex,
) -> BergResult<'a> {
    BergError::UnsupportedOperator(Box::new(operand.into()), Fixity::Postfix, operator).err()
}

pub fn default_prefix<'a, T: BergValue<'a> + Into<BergVal<'a>>>(
    operand: T,
    operator: IdentifierIndex,
) -> BergResult<'a> {
    use crate::syntax::identifiers::{DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT};
    match operator {
        DOUBLE_EXCLAMATION_POINT => operand.prefix(EXCLAMATION_POINT)?.prefix(EXCLAMATION_POINT),
        _ => {
            BergError::UnsupportedOperator(Box::new(operand.into()), Fixity::Prefix, operator).err()
        }
    }
}

pub fn default_field<'a, T: BergValue<'a> + Into<BergVal<'a>> + Clone>(
    object: &T,
    name: IdentifierIndex,
) -> BergResult<'a> {
    BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into()), name).err()
}

#[allow(clippy::needless_pass_by_value)]
pub fn default_set_field<'a, T: BergValue<'a> + Into<BergVal<'a>> + Clone>(
    object: &mut T,
    name: IdentifierIndex,
    _value: BergResult<'a>,
) -> BergResult<'a, ()> {
    BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into()), name).err()
}
