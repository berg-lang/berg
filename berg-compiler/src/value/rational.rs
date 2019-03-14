use crate::syntax::identifiers::*;
use crate::syntax::IdentifierIndex;
use crate::value::implement::*;
use num::{BigInt, BigRational, One, ToPrimitive, Zero};
use std::{i64, u64};

impl<'a> BergValue<'a> for BigRational {
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        single_next_val(self)
    }
    fn into_val(self) -> BergResult<'a> {
        self.ok()
    }
    fn eval_val(self) -> EvalResult<'a> {
        self.ok()
    }
    fn at_position(self, _new_position: ExpressionErrorPosition) -> BergResult<'a> {
        self.ok()
    }
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        default_into_native(self)
    }
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        default_try_into_native(self)
    }
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        match operator {
            PLUS => (self + right.into_native::<BigRational>()?).ok(),
            DASH => (self - right.into_native::<BigRational>()?).ok(),
            SLASH => {
                let right = right.into_native::<BigRational>()?;
                if right.is_zero() {
                    BergError::DivideByZero.err()
                } else {
                    (self / right).ok()
                }
            }
            STAR => (self * right.into_native::<BigRational>()?).ok(),
            EQUAL_TO => match right.try_into_native::<BigRational>()? {
                Some(right) => self == right,
                None => false,
            }.ok(),
            GREATER_THAN => (self > right.into_native::<BigRational>()?).ok(),
            LESS_THAN => (self < right.into_native::<BigRational>()?).ok(),
            GREATER_EQUAL => (self >= right.into_native::<BigRational>()?).ok(),
            LESS_EQUAL => (self <= right.into_native::<BigRational>()?).ok(),
            _ => default_infix(self, operator, right),
        }
    }

    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a> {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match operator {
            PLUS => (self).ok(),
            DASH => (-self).ok(),
            PLUS_ONE => (self + BigRational::one()).ok(),
            MINUS_ONE => (self - BigRational::one()).ok(),
            _ => default_prefix(self, operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> {
        match operator {
            PLUS_ONE => (self + BigRational::one()).ok(),
            MINUS_ONE => (self - BigRational::one()).ok(),
            _ => default_postfix(self, operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> {
        default_subexpression_result(self, boundary)
    }

    fn field(self, name: IdentifierIndex) -> EvalResult<'a> {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> {
        default_set_field(self, name, value)
    }
}

impl<'a> From<BigInt> for BergVal<'a> {
    fn from(from: BigInt) -> Self {
        BigRational::from(from).into()
    }
}
impl<'a> From<BigInt> for EvalVal<'a> {
    fn from(from: BigInt) -> Self {
        BergVal::from(from).into()
    }
}

impl<'a> From<BigRational> for BergVal<'a> {
    fn from(from: BigRational) -> Self {
        BergVal::BigRational(from)
    }
}
impl<'a> From<BigRational> for EvalVal<'a> {
    fn from(from: BigRational) -> Self {
        BergVal::from(from).into()
    }
}


impl<'a> TryFromBergVal<'a> for BigRational {
    const TYPE_NAME: &'static str = "number";
    fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, ErrorVal<'a>> {
        match from.into_val()? {
            BergVal::BigRational(value) => Ok(Ok(value)),
            from => Ok(Err(from))
        }
    }
}

macro_rules! impl_berg_val_for_primitive_num {
    ($($type:ty: $to:tt),*) => {
        $(
            impl<'a> TryFromBergVal<'a> for $type {
                const TYPE_NAME: &'static str = stringify!($type);
                fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, ErrorVal<'a>> {
                    match from.into_val()? {
                        BergVal::BigRational(value) => {
                            if value.is_integer() {
                                if let Some(value) = value.to_integer().$to() {
                                    return Ok(Ok(value));
                                }
                            }
                            Ok(Err(BergVal::BigRational(value)))
                        },
                        value => Ok(Err(value))
                    }
                }
            }

            impl<'a> From<$type> for BergVal<'a> {
                fn from(from: $type) -> Self {
                    BigInt::from(from).into()
                }
            }
            impl<'a> From<$type> for EvalVal<'a> {
                fn from(from: $type) -> Self {
                    BergVal::from(from).into()
                }
            }
        )*
    }
}

impl_berg_val_for_primitive_num! { u64: to_u64, i64: to_i64, u32: to_u32, i32: to_i32, u16: to_u16, i16: to_i16, u8: to_u8, i8: to_i8, usize: to_usize, isize: to_isize }
