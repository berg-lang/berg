use std::marker::PhantomData;
use crate::value::*;
use std::fmt;
use ExpressionErrorPosition::*;

///
/// A value that can participate in Berg expressions.
///
pub trait BergValue<'a>: Sized + fmt::Debug {
    ///
    /// Get the next value.
    /// 
    /// All BergValues are streams. This is the implementation. Returns:
    /// 
    /// - `Ok(None)` if there is no value.
    /// - `Ok(Some(NextVal { head, tail })) if there is a value.
    /// - `Err(error)` if we cannot tell whether there is a next value or not
    ///   due to an error.
    ///
    fn next_val(self) -> Result<Option<NextVal<'a>>, ErrorVal<'a>>;

    ///
    /// Get the result of this value as a particular native type.
    ///
    /// If it's a *block,* it evaluates the value and tries to convert it.
    ///
    /// If conversion fails, Err(BergError::BadOperandType(..)) is returned.
    /// If there is an error evaluating the value, it is returned.
    /// 
    /// Example:
    /// 
    ///     use berg_compiler::*;
    ///     fn add_values<'a>(a: impl BergValue<'a>, b: impl BergValue<'a>) -> Result<usize, ErrorVal<'a>> {
    ///         Ok(a.into_native::<usize>()? + b.into_native::<usize>()?)
    ///     }
    /// 
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>>;

    ///
    /// Get the result of this value as a particular native type.
    ///
    /// If it's a *block,* it evaluates the value and tries to convert it.
    ///
    /// If conversion succeeds, Ok(Some(value)) is returned.
    /// If conversion fails, Ok(None) is returned.
    /// If there is an error evaluating the value, the error is returned.
    /// 
    /// Example:
    /// 
    ///     use berg_compiler::*;
    ///     fn check_equal<'a>(a: usize, b: impl BergValue<'a>) -> Result<bool, ErrorVal<'a>> {
    ///         match b.try_into_native::<usize>()? {
    ///             Some(b) => Ok(a == b),
    ///             None => Ok(false),
    ///         }
    ///     }
    /// 
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>>;

    ///
    /// Get a concrete BergVal for this value.
    /// 
    fn into_val(self) -> BergResult<'a>;

    ///
    /// Get a concrete BergVal for this value, adjusted for the given expression position.
    /// 
    fn at_position(self, new_position: ExpressionErrorPosition) -> BergResult<'a>;

    ///
    /// Get a concrete EvalVal for this value.
    /// 
    fn eval_val(self) -> EvalResult<'a>;

    ///
    /// Evaluate this value immediately, even if it is lazy.
    /// 
    fn evaluate(self) -> BergResult<'a>;

    fn field(self, name: IdentifierIndex) -> EvalResult<'a>;
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), ErrorVal<'a>> where Self: Clone;

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a>;
    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> EvalResult<'a>;
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a>;
    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a>;
    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a>;
}

pub trait TryFromBergVal<'a>: Sized+fmt::Debug {
    const TYPE_NAME: &'static str;

    ///
    /// Try to convert 
    /// 
    /// Returns:
    /// - `Ok(Ok(value))` if the conversion succeeded.
    /// - `Err(error)` if there was an error calculating the value.
    /// - `Ok(Err(value))` if the value was calculated, but could not be converted to the native type.
    ///
    fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, ErrorVal<'a>>;
}

#[derive(Debug, Copy, Clone)]
pub struct RightOperand<'a, V: BergValue<'a>>(pub V, pub PhantomData<&'a ()>);

impl<'a, V: BergValue<'a>> From<V> for RightOperand<'a, V> {
    fn from(from: V) -> Self {
        RightOperand(from, PhantomData)
    }
}

impl<'a, V: BergValue<'a>> RightOperand<'a, V> {
    pub fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, ErrorVal<'a>> {
        match self.0.into_native() {
            Ok(value) => Ok(value),
            Err(error) => error.reposition(Right).err(),
        }
    }
    pub fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, ErrorVal<'a>> {
        match self.0.try_into_native() {
            Ok(value) => Ok(value),
            Err(error) => error.reposition(Right).err(),
        }
    }
    ///
    /// Get the operand's value with appropriate error locations and no
    /// EvalVal values.
    /// 
    pub fn into_val(self) -> BergResult<'a> {
        self.0.into_val().at_position(Right)
    }
    ///
    /// Process the value and give appropriate error locations to the result.
    /// 
    pub fn eval_val(self) -> Result<RightOperand<'a, EvalVal<'a>>, ErrorVal<'a>> {
        Ok(self.0.eval_val()?.into())
    }
    pub fn evaluate(self) -> BergResult<'a> {
        self.0.evaluate().at_position(Right)
    }
    ///
    /// Process the value and give appropriate error locations to the result.
    ///  
    pub fn get(self) -> Result<RightOperand<'a, EvalVal<'a>>, ErrorVal<'a>> {
        Ok(self.0.eval_val()?.get()?.into())
    }
}

impl<'a, T: BergValue<'a>> fmt::Display for RightOperand<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub mod implement {
    pub use crate::value::*;
    pub use crate::syntax::ExpressionRef;
    pub use crate::value::ExpressionErrorPosition::*;
    pub use crate::value::BergError::*;

    use crate::syntax::Fixity;

    pub fn single_next_val<'a>(value: impl BergValue<'a>) -> Result<Option<NextVal<'a>>, ErrorVal<'a>> {
        Ok(Some(NextVal::single(value.into_val()?)))
    }

    pub fn default_into_native<'a, T: TryFromBergVal<'a>>(value: impl BergValue<'a>) -> Result<T, ErrorVal<'a>>  {
        match T::try_from_berg_val(value.eval_val()?) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(original)) => BergError::BadOperandType(Box::new(Ok(original)), T::TYPE_NAME).err(),
            Err(error) => Err(error),
        }
    }

    pub fn default_try_into_native<'a, T: TryFromBergVal<'a>>(value: impl BergValue<'a>) -> Result<Option<T>, ErrorVal<'a>>  {
        match T::try_from_berg_val(value.eval_val()?) {
            Ok(Ok(value)) => Ok(Some(value)),
            Ok(Err(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn default_subexpression_result<'a>(value: impl BergValue<'a>, _boundary: ExpressionBoundary) -> EvalResult<'a> {
        value.eval_val()
    }

    pub fn default_infix<'a>(
        left: impl BergValue<'a>,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl BergValue<'a>>,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::{
            COLON, COMMA, DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE, NOT_EQUAL_TO, SEMICOLON,
        };
        match operator {
            COMMA => {
                let left = left.into_val()?;
                match right.eval_val()? {
                    // (1,)
                    RightOperand(EvalVal::MissingExpression, _) => EvalVal::TrailingComma(vec![left]).ok(),
                    // (1,2[,...])
                    right => EvalVal::PartialTuple(vec![left, right.into_val()?]).ok(),
                }
            }
            SEMICOLON => {
                left.evaluate()?;
                match right.eval_val()? {
                    RightOperand(EvalVal::MissingExpression, _) => EvalVal::TrailingSemicolon.ok(),
                    right => right.evaluate()?.ok(),
                }
            }
            NEWLINE => {
                left.evaluate()?;
                right.evaluate()?.ok()
            }
            EQUAL_TO => {
                let mut left_next = left.next_val()?;
                let mut right_next = right.into_val().next_val()?;
                loop {
                    match (left_next, right_next) {
                        (None, None) => return true.ok(),
                        (Some(_), None) | (None, Some(_)) => return false.ok(),
                        (Some(left), Some(right)) => {
                            println!("EQUAL {} == {}", left.head, right.head);
                            if left.head.infix(EQUAL_TO, right.head.into()).into_native::<bool>()? {
                                left_next = left.tail.next_val()?;
                                right_next = right.tail.next_val()?;
                            } else {
                                return false.ok();
                            }
                        }
                    }
                }
            }
            NOT_EQUAL_TO => left.infix(EQUAL_TO, right)?.prefix(EXCLAMATION_POINT),
            DOT => {
                let left = left.into_val()?;
                match right.try_into_native::<IdentifierIndex>()? {
                    Some(name) => AssignmentTarget::ObjectFieldReference(left, name).ok(),
                    None => BergError::RightSideOfDotMustBeIdentifier.operand_err(Right),
                }
            }
            COLON => BergError::AssignmentTargetMustBeIdentifier.operand_err(Left),
            _ => BergError::UnsupportedOperator(Box::new(left.into_val()), Fixity::Infix, operator).err(),
        }
    }

    pub fn default_infix_assign<'a>(
        _left: impl BergValue<'a>,
        _operator: IdentifierIndex,
        _right: RightOperand<'a, impl BergValue<'a>>,
    ) -> EvalResult<'a> {
        BergError::AssignmentTargetMustBeIdentifier.operand_err(Left)
    }

    pub fn default_postfix<'a>(
        operand: impl BergValue<'a>,
        operator: IdentifierIndex,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::{PLUS_PLUS, DASH_DASH};
        match operator {
            PLUS_PLUS | DASH_DASH => BergError::AssignmentTargetMustBeIdentifier.operand_err(Left),
            _ => BergError::UnsupportedOperator(Box::new(operand.into_val()), Fixity::Prefix, operator).err(),
        }
    }

    pub fn default_prefix<'a>(
        operand: impl BergValue<'a>,
        operator: IdentifierIndex,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::{DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT, PLUS_PLUS, DASH_DASH};
        match operator {
            DOUBLE_EXCLAMATION_POINT => operand.prefix(EXCLAMATION_POINT)?.prefix(EXCLAMATION_POINT),
            PLUS_PLUS | DASH_DASH => BergError::AssignmentTargetMustBeIdentifier.operand_err(Right),
            _ => BergError::UnsupportedOperator(Box::new(operand.into_val()), Fixity::Prefix, operator).err(),
        }
    }

    pub fn default_field<'a, T: BergValue<'a>>(
        object: T,
        name: IdentifierIndex,
    ) -> EvalResult<'a> {
        BergError::NoSuchPublicFieldOnValue(Box::new(object.into_val()), name).err()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn default_set_field<'a, T: BergValue<'a>+Clone>(
        object: &mut T,
        name: IdentifierIndex,
        _value: BergVal<'a>,
    ) -> Result<(), ErrorVal<'a>> {
        BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into_val()), name).err()
    }
}
