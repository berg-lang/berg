use crate::value::*;
use berg_parser::ExpressionPosition::*;
use std::fmt;

///
/// Berg values that can be used anywhere.
///
/// See also [`Val`], which includes values that need the expression
/// evaluator before they can be used.
///
pub trait BergValue:
    Value + IteratorValue + ObjectValue + OperableValue + BoxCloneBergValue
{
}

///
/// Values that can be used by the expression evaluator.
///
pub trait Value: fmt::Debug {
    ///
    /// Get the result of this value as a particular native type.
    ///
    /// If it's a *block,* it evaluates the value and tries to convert it.
    ///
    /// If conversion fails, Err(CompilerError::BadOperandType(..)) is returned.
    /// If there is an error evaluating the value, it is returned.
    ///
    /// Example:
    ///
    ///     use berg_compiler::*;
    ///     fn add_values(a: impl BergValue, b: impl BergValue) -> Result<usize, EvalException> {
    ///         Ok(a.into_native::<usize>()? + b.into_native::<usize>()?)
    ///     }
    ///
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException>
    where
        Self: Sized;

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
    ///     fn check_equal(a: usize, b: impl BergValue) -> Result<bool, EvalException> {
    ///         match b.try_into_native::<usize>()? {
    ///             Some(b) => Ok(a == b),
    ///             None => Ok(false),
    ///         }
    ///     }
    ///
    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException>
    where
        Self: Sized;

    ///
    /// Get a BergVal for this value, but don't necessarily evaluate it.
    ///
    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized;

    ///
    /// Get a concrete EvalVal for this value.
    ///
    fn eval_val(self) -> EvalResult
    where
        Self: Sized;

    ///
    /// Get a [`Display`]able version of this value.
    ///
    fn display(&self) -> &dyn fmt::Display;
}

pub trait EvaluatableValue: Value {
    ///
    /// Evaluate this value immediately, even if it is lazy.
    ///
    fn evaluate(self) -> BergResult
    where
        Self: Sized;
}

pub trait IteratorValue: Value {
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
    fn next_val(self) -> Result<NextVal, EvalException>
    where
        Self: Sized;
}

pub trait ObjectValue: Value {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized;
    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal,
    ) -> Result<(), EvalException>
    where
        Self: Clone;
}

pub trait OperableValue: Value {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized;
    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized;
    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized;
    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized;
    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized;
}

pub trait TryFromBergVal: fmt::Debug {
    const TYPE_NAME: &'static str;

    ///
    /// Try to convert
    ///
    /// Returns:
    /// - `Ok(Ok(value))` if the conversion succeeded.
    /// - `Err(error)` if there was an error calculating the value.
    /// - `Ok(Err(value))` if the value was calculated, but could not be converted to the native type.
    ///
    fn try_from_berg_val(from: EvalVal) -> Result<Result<Self, BergVal>, EvalException>
    where
        Self: Sized;
}

///
/// Holds the right operand to an infix operator.
///
/// `RightOperand`'s primary purpose is to ensure exceptions thrown when
/// evaluating the right operand are automatically given the correct error range.
///
#[derive(Debug, Copy, Clone)]
pub struct RightOperand<V: Value>(pub V);

///
/// The result of [`IteratorValue.next_val()`].
///
#[derive(Debug, Clone)]
pub struct NextVal {
    ///
    /// The next value, or `None` if there is no next value.
    ///
    pub head: Option<BergVal>,
    ///
    /// The next iterator (may be an empty value if there is nothing to do).
    ///
    pub tail: BergVal,
}

///
/// Allows us to hold boxed `BergValue`s in cloneable objects.
///
pub trait BoxCloneBergValue {
    ///
    /// Clone this BergValue into a Box.
    ///
    fn box_clone(&self) -> Box<dyn BergValue>;
}

impl<T: BergValue + Clone + 'static> BoxCloneBergValue for T {
    fn box_clone(&self) -> Box<dyn BergValue> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn BergValue> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl<V: Value> From<V> for RightOperand<V> {
    fn from(from: V) -> Self {
        RightOperand(from)
    }
}

impl<V: EvaluatableValue> RightOperand<V> {
    pub fn evaluate(self) -> BergResult {
        self.0.evaluate()
    }
}

impl<V: Value> RightOperand<V> {
    pub fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        match self.0.into_native() {
            Ok(value) => Ok(value),
            Err(error) => error.reposition(Right).err(),
        }
    }
    pub fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        match self.0.try_into_native() {
            Ok(value) => Ok(value),
            Err(error) => error.reposition(Right).err(),
        }
    }
    ///
    /// Get the operand's value with appropriate error locations and no
    /// EvalVal values.
    ///
    pub fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.0.lazy_val().map_err(|e| e.reposition(Right))
    }
    ///
    /// Process the value and give appropriate error locations to the result.
    ///
    pub fn eval_val(self) -> Result<RightOperand<EvalVal>, EvalException> {
        Ok(self.0.eval_val()?.into())
    }
    ///
    /// Process the value and give appropriate error locations to the result.
    ///  
    pub fn get(self) -> Result<RightOperand<EvalVal>, EvalException> {
        Ok(self.0.eval_val()?.get()?.into())
    }
}

impl<T: Value> fmt::Display for RightOperand<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for NextVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.head {
            None => write!(f, "<none> -> {}", self.tail.display()),
            Some(ref value) => write!(f, "{} -> {}", value, self.tail.display()),
        }
    }
}
impl NextVal {
    pub fn none(iterator: BergVal) -> NextVal {
        NextVal {
            head: None,
            tail: iterator,
        }
    }
    pub fn single(value: BergVal) -> NextVal {
        NextVal {
            head: Some(value),
            tail: empty_tuple(),
        }
    }
}

pub mod implement {
    pub use super::expression::ExpressionRef;
    pub use crate::value::CompilerError::*;
    pub use crate::value::*;
    pub use berg_parser::ExpressionPosition::*;

    use berg_parser::Fixity;

    pub fn single_next_val(value: impl Value) -> Result<NextVal, EvalException> {
        NextVal::single(value.lazy_val()?).ok()
    }

    pub fn default_into_native<T: TryFromBergVal>(
        value: impl Value,
    ) -> Result<T, EvalException> {
        match T::try_from_berg_val(value.eval_val()?) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(original)) => {
                CompilerError::BadOperandType(Box::new(original), T::TYPE_NAME).err()
            }
            Err(error) => Err(error),
        }
    }

    pub fn default_try_into_native<T: TryFromBergVal>(
        value: impl Value,
    ) -> Result<Option<T>, EvalException> {
        match T::try_from_berg_val(value.eval_val()?) {
            Ok(Ok(value)) => Ok(Some(value)),
            Ok(Err(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn default_subexpression_result(
        value: impl Value,
        _boundary: ExpressionBoundary,
    ) -> EvalResult {
        value.eval_val()
    }

    pub fn default_infix(
        left: impl OperableValue + IteratorValue,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult {
        use berg_parser::identifiers::{
            COLON, COMMA, DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE_SEQUENCE, NOT_EQUAL_TO,
            SEMICOLON,
        };
        match operator {
            COMMA => {
                let left = left.lazy_val()?;
                match right.eval_val()? {
                    // (1,)
                    RightOperand(EvalVal::MissingExpression) => {
                        EvalVal::TrailingComma(vec![left]).ok()
                    }
                    // (1,2[,...])
                    right => EvalVal::PartialTuple(vec![left, right.lazy_val()?]).ok(),
                }
            }
            SEMICOLON => {
                left.lazy_val()?.evaluate()?;
                match right.eval_val()? {
                    RightOperand(EvalVal::MissingExpression) => EvalVal::TrailingSemicolon.ok(),
                    right => right.lazy_val()?.evaluate()?.ok(),
                }
            }
            NEWLINE_SEQUENCE => {
                left.lazy_val()?.evaluate()?;
                right.lazy_val()?.evaluate()?.ok()
            }
            EQUAL_TO => {
                let mut left_next = left.next_val()?;
                let mut right_next = right.lazy_val().next_val()?;
                loop {
                    match (left_next.head, right_next.head) {
                        (None, None) => return true.ok(),
                        (Some(_), None) | (None, Some(_)) => return false.ok(),
                        (Some(left), Some(right)) => {
                            println!("EQUAL {} == {}", left, right);
                            if left.infix(EQUAL_TO, right.into()).into_native::<bool>()? {
                                left_next = left_next.tail.next_val()?;
                                right_next = right_next.tail.next_val()?;
                            } else {
                                return false.ok();
                            }
                        }
                    }
                }
            }
            NOT_EQUAL_TO => left.infix(EQUAL_TO, right)?.prefix(EXCLAMATION_POINT),
            DOT => {
                let left = left.lazy_val()?;
                match right.try_into_native::<IdentifierIndex>()? {
                    Some(name) => AssignmentTarget::ObjectFieldReference(left, name).ok(),
                    None => CompilerError::RightSideOfDotMustBeIdentifier.operand_err(Right),
                }
            }
            COLON => CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Left),
            _ => CompilerError::UnsupportedOperator(
                Box::new(left.lazy_val()?),
                Fixity::Infix,
                operator,
            )
            .err(),
        }
    }

    pub fn default_infix_assign(
        _left: impl Value,
        _operator: IdentifierIndex,
        _right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult {
        CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Left)
    }

    pub fn default_postfix(
        operand: impl Value,
        operator: IdentifierIndex,
    ) -> EvalResult {
        use berg_parser::identifiers::{DASH_DASH, PLUS_PLUS};
        match operator {
            PLUS_PLUS | DASH_DASH => {
                CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Left)
            }
            _ => CompilerError::UnsupportedOperator(
                Box::new(operand.lazy_val()?),
                Fixity::Prefix,
                operator,
            )
            .err(),
        }
    }

    pub fn default_prefix(
        operand: impl OperableValue,
        operator: IdentifierIndex,
    ) -> EvalResult {
        use berg_parser::identifiers::{
            DASH_DASH, DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT, PLUS_PLUS,
        };
        match operator {
            DOUBLE_EXCLAMATION_POINT => {
                operand.prefix(EXCLAMATION_POINT)?.prefix(EXCLAMATION_POINT)
            }
            PLUS_PLUS | DASH_DASH => {
                CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Right)
            }
            _ => CompilerError::UnsupportedOperator(
                Box::new(operand.lazy_val()?),
                Fixity::Prefix,
                operator,
            )
            .err(),
        }
    }

    pub fn default_field(object: impl Value, name: IdentifierIndex) -> EvalResult {
        CompilerError::NoSuchPublicFieldOnValue(Box::new(object.lazy_val()?), name).err()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn default_set_field(
        object: &mut (impl Value + Clone),
        name: IdentifierIndex,
        _value: BergVal,
    ) -> Result<(), EvalException> {
        CompilerError::NoSuchPublicFieldOnValue(Box::new(object.clone().lazy_val()?), name).err()
    }
}
