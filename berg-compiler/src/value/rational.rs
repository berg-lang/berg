use crate::value::implement::*;
use berg_parser::identifiers::*;
use berg_parser::IdentifierIndex;
use num::{BigInt, BigRational, One, ToPrimitive, Zero};
use std::{i64, u64};

impl BergValue for BigRational {}

impl EvaluatableValue for BigRational {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.ok()
    }
}

impl Value for BigRational {
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.ok()
    }
    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        self.ok()
    }
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        default_into_native(self)
    }
    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        default_try_into_native(self)
    }
    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl IteratorValue for BigRational {
    fn next_val(self) -> Result<NextVal, EvalException> {
        single_next_val(self)
    }
}

impl ObjectValue for BigRational {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        default_field(self, name)
    }
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal) -> Result<(), EvalException> {
        default_set_field(self, name, value)
    }
}

impl OperableValue for BigRational {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        match operator {
            PLUS => (self + right.into_native::<BigRational>()?).ok(),
            DASH => (self - right.into_native::<BigRational>()?).ok(),
            SLASH => {
                let right = right.into_native::<BigRational>()?;
                if right.is_zero() {
                    CompilerError::DivideByZero.operand_err(Right)
                } else {
                    (self / right).ok()
                }
            }
            STAR => (self * right.into_native::<BigRational>()?).ok(),
            EQUAL_TO => match right.try_into_native::<BigRational>()? {
                Some(right) => self == right,
                None => false,
            }
            .ok(),
            GREATER_THAN => (self > right.into_native::<BigRational>()?).ok(),
            LESS_THAN => (self < right.into_native::<BigRational>()?).ok(),
            GREATER_EQUAL => (self >= right.into_native::<BigRational>()?).ok(),
            LESS_EQUAL => (self <= right.into_native::<BigRational>()?).ok(),
            _ => default_infix(self, operator, right),
        }
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        default_infix_assign(self, operator, right)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match operator {
            PLUS => (self).ok(),
            DASH => (-self).ok(),
            PLUS_ONE => (self + BigRational::one()).ok(),
            MINUS_ONE => (self - BigRational::one()).ok(),
            _ => default_prefix(self, operator),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        match operator {
            PLUS_ONE => (self + BigRational::one()).ok(),
            MINUS_ONE => (self - BigRational::one()).ok(),
            _ => default_postfix(self, operator),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        default_subexpression_result(self, boundary)
    }
}

impl From<BigInt> for BergVal {
    fn from(from: BigInt) -> Self {
        BigRational::from(from).into()
    }
}
impl From<BigInt> for EvalVal {
    fn from(from: BigInt) -> Self {
        BergVal::from(from).into()
    }
}

impl From<BigRational> for BergVal {
    fn from(from: BigRational) -> Self {
        BergVal::BigRational(from)
    }
}
impl From<BigRational> for EvalVal {
    fn from(from: BigRational) -> Self {
        BergVal::from(from).into()
    }
}

impl TryFromBergVal for BigRational {
    const TYPE_NAME: &'static str = "number";
    fn try_from_berg_val(from: EvalVal) -> Result<Result<Self, BergVal>, EvalException> {
        match from.lazy_val()? {
            BergVal::BigRational(value) => Ok(Ok(value)),
            from => Ok(Err(from)),
        }
    }
}

macro_rules! impl_berg_val_for_primitive_num {
    ($($type:ty: $to:tt),*) => {
        $(
            impl TryFromBergVal for $type {
                const TYPE_NAME: &'static str = stringify!($type);
                fn try_from_berg_val(from: EvalVal) -> Result<Result<Self, BergVal>, EvalException> {
                    match from.lazy_val()? {
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

            impl From<$type> for BergVal {
                fn from(from: $type) -> Self {
                    BigInt::from(from).into()
                }
            }
            impl From<$type> for EvalVal {
                fn from(from: $type) -> Self {
                    BergVal::from(from).into()
                }
            }
        )*
    }
}

impl_berg_val_for_primitive_num! { u64: to_u64, i64: to_i64, u32: to_u32, i32: to_i32, u16: to_u16, i16: to_i16, u8: to_u8, i8: to_i8, usize: to_usize, isize: to_isize }
