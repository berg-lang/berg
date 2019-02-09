use crate::syntax::identifiers::*;
use crate::syntax::IdentifierIndex;
use crate::util::try_from::TryFrom;
use crate::util::type_name::TypeName;
use crate::value::*;
use num::{BigInt, BigRational, One, ToPrimitive, Zero};
use std::{i64, u64};

impl TypeName for BigRational {
    const TYPE_NAME: &'static str = "number";
}

impl<'a> BergValue<'a> for BigRational {
    fn infix<T: BergValue<'a>>(self, operator: IdentifierIndex, right: T) -> EvalResult<'a> {
        match operator {
            PLUS => (self + right.into_native::<BigRational>()??).ok(),
            DASH => (self - right.into_native::<BigRational>()??).ok(),
            SLASH => {
                let right = right.into_native::<BigRational>()??;
                if right.is_zero() {
                    BergError::DivideByZero.err()
                } else {
                    (self / right).ok()
                }
            }
            STAR => (self * right.into_native::<BigRational>()??).ok(),
            EQUAL_TO => match right.into_native::<BigRational>()? {
                Ok(right) => self == right,
                Err(_) => false,
            }
            .ok(),
            GREATER_THAN => (self > right.into_native::<BigRational>()??).ok(),
            LESS_THAN => (self < right.into_native::<BigRational>()??).ok(),
            GREATER_EQUAL => (self >= right.into_native::<BigRational>()??).ok(),
            LESS_EQUAL => (self <= right.into_native::<BigRational>()??).ok(),
            _ => default_infix(self, operator, right),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match operator {
            PLUS => (self).ok(),
            DASH => (-self).ok(),
            PLUS_PLUS => (self + BigRational::one()).ok(),
            DASH_DASH => (self - BigRational::one()).ok(),
            _ => default_prefix(self, operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match operator {
            PLUS_PLUS => (self + BigRational::one()).ok(),
            DASH_DASH => (self - BigRational::one()).ok(),
            _ => default_postfix(self, operator),
        }
    }

    fn field(&self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> EvalResult<'a, ()> {
        default_set_field(self, name, value)
    }
    fn into_val(self) -> BergResult<'a> {
        Ok(self.into())
    }
    fn next_val(self) -> BergResult<'a, NextVal<'a>> {
        Ok(NextVal::single(self.into()))
    }
}

impl<'a> From<BigInt> for BergVal<'a> {
    fn from(from: BigInt) -> Self {
        BigRational::from(from).into()
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
