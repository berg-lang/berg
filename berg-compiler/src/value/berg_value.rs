use std::marker::PhantomData;
use crate::value::*;
use std::fmt;
use ExpressionErrorPosition::*;

///
/// Berg values that can be used anywhere.
/// 
/// See also [`Val`], which includes values that need the expression
/// evaluator before they can be used.
/// 
pub trait BergValue<'a>: Value<'a>+IteratorValue<'a>+ObjectValue<'a>+OperableValue<'a>+BoxCloneBergValue<'a> {
}

///
/// Values that can be used by the expression evaluator.
///
pub trait Value<'a>: fmt::Debug {
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
    ///     fn add_values<'a>(a: impl BergValue<'a>, b: impl BergValue<'a>) -> Result<usize, EvalException<'a>> {
    ///         Ok(a.into_native::<usize>()? + b.into_native::<usize>()?)
    ///     }
    /// 
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> where Self: Sized;

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
    ///     fn check_equal<'a>(a: usize, b: impl BergValue<'a>) -> Result<bool, EvalException<'a>> {
    ///         match b.try_into_native::<usize>()? {
    ///             Some(b) => Ok(a == b),
    ///             None => Ok(false),
    ///         }
    ///     }
    /// 
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> where Self: Sized;

    ///
    /// Get a BergVal for this value, but don't necessarily evaluate it.
    /// 
    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized;

    ///
    /// Get a concrete EvalVal for this value.
    /// 
    fn eval_val(self) -> EvalResult<'a> where Self: Sized;

    ///
    /// Get a [`Display`]able version of this value.
    /// 
    fn display(&self) -> &fmt::Display;
}

pub trait EvaluatableValue<'a>: Value<'a> {
    ///
    /// Evaluate this value immediately, even if it is lazy.
    /// 
    fn evaluate(self) -> BergResult<'a> where Self: Sized;
}

pub trait IteratorValue<'a>: Value<'a> {
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
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> where Self: Sized;
}

pub trait ObjectValue<'a>: Value<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a> where Self: Sized;
    fn set_field(&mut self, name: IdentifierIndex, value: BergVal<'a>) -> Result<(), EvalException<'a>> where Self: Clone;
}

pub trait OperableValue<'a>: Value<'a> {
    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized;
    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl EvaluatableValue<'a>>) -> EvalResult<'a> where Self: Sized;
    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized;
    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a> where Self: Sized;
    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a> where Self: Sized;
}

pub trait TryFromBergVal<'a>: fmt::Debug {
    const TYPE_NAME: &'static str;

    ///
    /// Try to convert 
    /// 
    /// Returns:
    /// - `Ok(Ok(value))` if the conversion succeeded.
    /// - `Err(error)` if there was an error calculating the value.
    /// - `Ok(Err(value))` if the value was calculated, but could not be converted to the native type.
    ///
    fn try_from_berg_val(from: EvalVal<'a>) -> Result<Result<Self, BergVal<'a>>, EvalException<'a>> where Self: Sized;
}

///
/// Holds the right operand to an infix operator.
/// 
/// `RightOperand`'s primary purpose is to ensure exceptions thrown when
/// evaluating the right operand are automatically given the correct error range.
/// 
#[derive(Debug, Copy, Clone)]
pub struct RightOperand<'a, V: Value<'a>>(pub V, pub PhantomData<&'a ()>);

///
/// The result of [`IteratorValue.next_val()`].
/// 
#[derive(Debug, Clone)]
pub struct NextVal<'a> {
    ///
    /// The next value, or `None` if there is no next value.
    /// 
    pub head: Option<BergVal<'a>>,
    ///
    /// The next iterator (may be an empty value if there is nothing to do).
    /// 
    pub tail: BergVal<'a>,
}

///
/// Allows us to hold boxed `BergValue`s in cloneable objects.
/// 
pub trait BoxCloneBergValue<'a> {
    ///
    /// Clone this BergValue into a Box.
    /// 
    fn box_clone(&self) -> Box<BergValue<'a>+'a>;
}

impl<'a, T: BergValue<'a>+Clone+'a> BoxCloneBergValue<'a> for T {
    fn box_clone(&self) -> Box<BergValue<'a>+'a> {
        Box::new(self.clone())
    }
}

impl<'a> Clone for Box<BergValue<'a>+'a> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl<'a, V: Value<'a>> From<V> for RightOperand<'a, V> {
    fn from(from: V) -> Self {
        RightOperand(from, PhantomData)
    }
}

impl<'a, V: EvaluatableValue<'a>> RightOperand<'a, V> {
    pub fn evaluate(self) -> BergResult<'a> {
        self.0.evaluate()
    }
}

impl<'a, V: Value<'a>> RightOperand<'a, V> {
    pub fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        match self.0.into_native() {
            Ok(value) => Ok(value),
            Err(error) => error.reposition(Right).err(),
        }
    }
    pub fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        match self.0.try_into_native() {
            Ok(value) => Ok(value),
            Err(error) => error.reposition(Right).err(),
        }
    }
    ///
    /// Get the operand's value with appropriate error locations and no
    /// EvalVal values.
    /// 
    pub fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>> where Self: Sized {
        self.0.lazy_val().map_err(|e| e.reposition(Right))
    }
    ///
    /// Process the value and give appropriate error locations to the result.
    /// 
    pub fn eval_val(self) -> Result<RightOperand<'a, EvalVal<'a>>, EvalException<'a>> {
        Ok(self.0.eval_val()?.into())
    }
    ///
    /// Process the value and give appropriate error locations to the result.
    ///  
    pub fn get(self) -> Result<RightOperand<'a, EvalVal<'a>>, EvalException<'a>> {
        Ok(self.0.eval_val()?.get()?.into())
    }
}

impl<'a, T: Value<'a>> fmt::Display for RightOperand<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a> fmt::Display for NextVal<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.head {
            None => write!(f, "<none> -> {}", self.tail.display()),
            Some(ref value) => write!(f, "{} -> {}", value, self.tail.display()),
        }        
    }
}
impl<'a> NextVal<'a> {
    pub fn none(iterator: BergVal<'a>) -> NextVal<'a> {
        NextVal { head: None, tail: iterator}
    }
    pub fn single(value: BergVal<'a>) -> NextVal<'a> {
        NextVal { head: Some(value), tail: empty_tuple() }
    }
}

pub mod implement {
    pub use crate::value::*;
    pub use crate::syntax::ExpressionRef;
    pub use crate::value::ExpressionErrorPosition::*;
    pub use crate::value::CompilerError::*;

    use crate::syntax::Fixity;

    pub fn single_next_val<'a>(value: impl Value<'a>) -> Result<NextVal<'a>, EvalException<'a>> {
        NextVal::single(value.lazy_val()?).ok()
    }

    pub fn default_into_native<'a, T: TryFromBergVal<'a>>(value: impl Value<'a>) -> Result<T, EvalException<'a>>  {
        match T::try_from_berg_val(value.eval_val()?) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(original)) => CompilerError::BadOperandType(Box::new(original), T::TYPE_NAME).err(),
            Err(error) => Err(error),
        }
    }

    pub fn default_try_into_native<'a, T: TryFromBergVal<'a>>(value: impl Value<'a>) -> Result<Option<T>, EvalException<'a>>  {
        match T::try_from_berg_val(value.eval_val()?) {
            Ok(Ok(value)) => Ok(Some(value)),
            Ok(Err(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn default_subexpression_result<'a>(value: impl Value<'a>, _boundary: ExpressionBoundary) -> EvalResult<'a> {
        value.eval_val()
    }

    pub fn default_infix<'a>(
        left: impl Value<'a>+OperableValue<'a>+IteratorValue<'a>,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::{
            COLON, COMMA, DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE_SEQUENCE, NOT_EQUAL_TO, SEMICOLON,
        };
        match operator {
            COMMA => {
                let left = left.lazy_val()?;
                match right.eval_val()? {
                    // (1,)
                    RightOperand(EvalVal::MissingExpression, _) => EvalVal::TrailingComma(vec![left]).ok(),
                    // (1,2[,...])
                    right => EvalVal::PartialTuple(vec![left, right.lazy_val()?]).ok(),
                }
            }
            SEMICOLON => {
                left.lazy_val()?.evaluate()?;
                match right.eval_val()? {
                    RightOperand(EvalVal::MissingExpression, _) => EvalVal::TrailingSemicolon.ok(),
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
            _ => CompilerError::UnsupportedOperator(Box::new(left.lazy_val()?), Fixity::Infix, operator).err(),
        }
    }

    pub fn default_infix_assign<'a>(
        _left: impl Value<'a>,
        _operator: IdentifierIndex,
        _right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a> {
        CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Left)
    }

    pub fn default_postfix<'a>(
        operand: impl Value<'a>,
        operator: IdentifierIndex,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::{PLUS_PLUS, DASH_DASH};
        match operator {
            PLUS_PLUS | DASH_DASH => CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Left),
            _ => CompilerError::UnsupportedOperator(Box::new(operand.lazy_val()?), Fixity::Prefix, operator).err(),
        }
    }

    pub fn default_prefix<'a>(
        operand: impl Value<'a>+OperableValue<'a>+'a,
        operator: IdentifierIndex,
    ) -> EvalResult<'a> {
        use crate::syntax::identifiers::{DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT, PLUS_PLUS, DASH_DASH};
        match operator {
            DOUBLE_EXCLAMATION_POINT => operand.prefix(EXCLAMATION_POINT)?.prefix(EXCLAMATION_POINT),
            PLUS_PLUS | DASH_DASH => CompilerError::AssignmentTargetMustBeIdentifier.operand_err(Right),
            _ => CompilerError::UnsupportedOperator(Box::new(operand.lazy_val()?), Fixity::Prefix, operator).err(),
        }
    }

    pub fn default_field<'a>(
        object: impl Value<'a>+'a,
        name: IdentifierIndex,
    ) -> EvalResult<'a> {
        CompilerError::NoSuchPublicFieldOnValue(Box::new(object.lazy_val()?), name).err()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn default_set_field<'a>(
        object: &mut (impl Value<'a>+Clone),
        name: IdentifierIndex,
        _value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>> {
        CompilerError::NoSuchPublicFieldOnValue(Box::new(object.clone().lazy_val()?), name).err()
    }
}
