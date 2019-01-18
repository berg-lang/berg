use crate::error::{BergError, BergResult, EvalResult};
use crate::eval::{ScopeRef};
use crate::syntax::identifiers::*;
use crate::syntax::{AstRef, IdentifierIndex, Operand};
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::*;
use num::{BigInt, BigRational, One, ToPrimitive, Zero};
use std::{i64, u64};

impl TypeName for BigRational {
    const TYPE_NAME: &'static str = "number";
}

impl<'a> BergValue<'a> for BigRational {
    fn infix(
        self,
        operator: IdentifierIndex,
        scope: &mut ScopeRef<'a>,
        right: Operand,
        ast: &AstRef<'a>,
    ) -> EvalResult<'a> {
        match operator {
            PLUS => (self + right.result_to::<BigRational>(scope, ast)?).ok(),
            DASH => (self - right.result_to::<BigRational>(scope, ast)?).ok(),
            SLASH => {
                let right = right.result_to::<BigRational>(scope, ast)?;
                if right.is_zero() {
                    BergError::DivideByZero.err()
                } else {
                    (self / right).ok()
                }
            }
            STAR => (self * right.result_to::<BigRational>(scope, ast)?).ok(),
            EQUAL_TO => match right.result(scope, ast)?.downcast::<BigRational>() {
                Ok(ref value) if self == *value => true.ok(),
                _ => false.ok(),
            },
            GREATER_THAN => (self > right.result_to::<BigRational>(scope, ast)?).ok(),
            LESS_THAN => (self < right.result_to::<BigRational>(scope, ast)?).ok(),
            GREATER_EQUAL => (self >= right.result_to::<BigRational>(scope, ast)?).ok(),
            LESS_EQUAL => (self <= right.result_to::<BigRational>(scope, ast)?).ok(),
            _ => default_infix(self, operator, scope, right, ast),
        }
    }

    fn prefix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match operator {
            PLUS => (self).ok(),
            DASH => (-self).ok(),
            PLUS_PLUS => (self + BigRational::one()).ok(),
            DASH_DASH => (self - BigRational::one()).ok(),
            _ => default_prefix(self, operator, scope),
        }
    }

    fn postfix(self, operator: IdentifierIndex, scope: &mut ScopeRef<'a>) -> EvalResult<'a> {
        match operator {
            PLUS_PLUS => (self + BigRational::one()).ok(),
            DASH_DASH => (self - BigRational::one()).ok(),
            _ => default_postfix(self, operator, scope),
        }
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

impl<'a> From<BigInt> for BergVal<'a> {
    fn from(from: BigInt) -> Self {
        from.into()
    }
}

impl<'a> From<BigRational> for BergVal<'a> {
    fn from(from: BigRational) -> Self {
        BergVal::BigRational(from)
    }
}

impl<'a> TryFrom<BergVal<'a>> for BigRational {
    type Error = BergVal<'a>;
    fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
        match from {
            BergVal::BigRational(value) => Ok(value),
            _ => Err(from),
        }
    }
}

macro_rules! impl_berg_val_for_primitive_num {
    ($($type:ty: $to:tt),*) => {
        $(
            impl TypeName for $type {
                const TYPE_NAME: &'static str = stringify!($type);
            }

            impl<'a> From<$type> for BergVal<'a> {
                fn from(from: $type) -> Self {
                    BigInt::from(from).into()
                }
            }

            impl<'a> TryFrom<BergVal<'a>> for $type {
                type Error = BergVal<'a>;
                fn try_from(from: BergVal<'a>) -> Result<Self, Self::Error> {
                    match from {
                        BergVal::BigRational(ref value) if value.is_integer() => if let Some(value) = value.to_integer().$to() {
                            return Ok(value)
                        },
                        _ => {}
                    }
                    Err(from)
                }
            }
        )*
    }
}

impl_berg_val_for_primitive_num! { u64: to_u64, i64: to_i64, u32: to_u32, i32: to_i32, u16: to_u16, i16: to_i16, u8: to_u8, i8: to_i8 }
