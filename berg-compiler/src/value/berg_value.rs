use std::marker::PhantomData;
use crate::value::*;
use std::fmt;

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
    fn next_val(self) -> BergResult<'a, Option<NextVal<'a>>>;

    ///
    /// Get the result of this value as a particular native type.
    ///
    /// If it's a *block,* it evaluates the value and tries to convert it.
    ///
    /// If conversion fails, Err(BergError::BadType(..)) is returned.
    /// If there is an error evaluating the value, it is returned.
    /// 
    /// Example:
    /// 
    ///     fn add_values(a: impl BergValue<'a>, b: impl BergValue<'a>) -> BergResult<'a, usize> {
    ///         a.into_native::<usize>()? + b.into_native::<usize>()?
    ///     }
    /// 
    fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T>;

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
    ///     fn check_equal(a: u32, b: impl BergValue<'a>) -> BergResult<'a, usize> {
    ///         match b.try_into_native::<usize>()? {
    ///             Some(b) => a == b,
    ///             None => false,
    ///         }
    ///     }
    /// 
    fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>>;

    ///
    /// Get a concrete BergResult for this value.
    /// 
    /// This will not perform any evaluation or calculation.
    /// 
    fn into_result(self) -> BergResult<'a>;

    ///
    /// Convert this value into a BergResult.
    ///
    /// This does no evaluation and exists as a convenience for things like booleans.
    /// 
    fn ok<T: From<Self>, E>(self) -> Result<T, E> {
        Ok(T::from(self))
    }

    ///
    /// Convert this value into a BergResult.
    ///
    /// This does no evaluation and exists as a convenience for things like errors.
    /// 
    fn err<T, E: From<Self>>(self) -> Result<T, E> {
        Err(E::from(self))
    }

    fn field(self, name: IdentifierIndex) -> BergResult<'a>;
    fn set_field(&mut self, name: IdentifierIndex, value: BergResult<'a>) -> BergResult<'a, ()> where Self: Clone;

    fn infix(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a>;
    fn infix_assign(self, operator: IdentifierIndex, right: RightOperand<'a, impl BergValue<'a>>) -> BergResult<'a>;
    fn prefix(self, operator: IdentifierIndex) -> BergResult<'a>;
    fn postfix(self, operator: IdentifierIndex) -> BergResult<'a>;
    fn subexpression_result(self, boundary: ExpressionBoundary) -> BergResult<'a>;
    fn into_right_operand(self) -> BergResult<'a>;
}

pub trait TryFromBergVal<'a>: Sized {
    const TYPE_NAME: &'static str;

    ///
    /// Try to convert 
    /// 
    /// Returns:
    /// - `Ok(Ok(value))` if the conversion succeeded.
    /// - `Err(error)` if there was an error calculating the value.
    /// - `Ok(Err(value))` if the value was calculated, but could not be converted to the native type.
    ///
    fn try_from_berg_val(from: BergResult<'a>) -> BergResult<'a, Result<Self, BergVal<'a>>>;
}

#[derive(Debug, Copy, Clone)]
pub struct RightOperand<'a, V: BergValue<'a>>(V, PhantomData<&'a ()>);

impl<'a, V: BergValue<'a>> RightOperand<'a, V> {
    pub fn new(value: V) -> Self {
        RightOperand(value, PhantomData)
    }
    pub fn into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, T> {
        self.0.into_right_operand().into_native()
    }
    pub fn try_into_native<T: TryFromBergVal<'a>>(self) -> BergResult<'a, Option<T>> {
        self.0.into_right_operand().try_into_native()
    }
    pub fn into_result(self) -> BergResult<'a> {
        self.0.into_right_operand().into_result()
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
    use crate::syntax::Fixity;
    use crate::eval::AmbiguousSyntax;

    pub fn single_next_val<'a>(value: impl BergValue<'a>) -> BergResult<'a, Option<NextVal<'a>>> {
        Ok(Some(NextVal::single(value.into_result())))
    }

    pub fn default_into_native<'a, T: TryFromBergVal<'a>>(value: impl BergValue<'a>) -> BergResult<'a, T>  {
        match T::try_from_berg_val(value.into_result()) {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(original)) => BergError::BadType(Box::new(Ok(original)), T::TYPE_NAME).err(),
            Err(error) => Err(error),
        }
    }

    pub fn default_try_into_native<'a, T: TryFromBergVal<'a>>(value: impl BergValue<'a>) -> BergResult<'a, Option<T>>  {
        match T::try_from_berg_val(value.into_result()) {
            Ok(Ok(value)) => Ok(Some(value)),
            Ok(Err(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn default_subexpression_result<'a>(value: impl BergValue<'a>, _boundary: ExpressionBoundary) -> BergResult<'a> {
        value.into_result()
    }

    pub fn default_into_right_operand<'a>(value: impl BergValue<'a>) -> BergResult<'a> {
        value.into_result()
    }

    pub fn default_infix<'a>(
        left: impl BergValue<'a>,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl BergValue<'a>>,
    ) -> BergResult<'a> {
        use crate::syntax::identifiers::{
            COMMA, DOT, EQUAL_TO, EXCLAMATION_POINT, NEWLINE, NOT_EQUAL_TO, SEMICOLON,
        };
        match operator {
            COMMA => {
                let left = left.into_result()?;
                match right.into_result() {
                    Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::MissingExpression)) => AmbiguousSyntax::TrailingComma(vec![left]).err(),
                    right => AmbiguousSyntax::PartialTuple(vec![left, right?]).err(),
                }
            }
            SEMICOLON => match right.into_result() {
                Err(ControlVal::AmbiguousSyntax(AmbiguousSyntax::MissingExpression)) => AmbiguousSyntax::TrailingSemicolon.err(),
                right => right
            }
            NEWLINE => right.into_result(),
            EQUAL_TO => {
                let mut left_next = left.next_val()?;
                let mut right_next = right.into_result().next_val()?;
                loop {
                    match (left_next, right_next) {
                        (None, None) => return true.ok(),
                        (Some(_), None) | (None, Some(_)) => return false.ok(),
                        (Some(left), Some(right)) => {
                            if left.head.infix(EQUAL_TO, RightOperand::new(right.head)).into_native::<bool>()? {
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
                use crate::eval::AssignmentTarget;
                let left = left.into_result()?;
                match right.try_into_native::<IdentifierIndex>()? {
                    Some(name) => AssignmentTarget::ObjectFieldReference(left, name).result(),
                    None => BergError::RightSideOfDotMustBeIdentifier.err(),
                }
            }
            _ => BergError::UnsupportedOperator(Box::new(left.into_result()), Fixity::Infix, operator).err(),
        }
    }

    pub fn default_infix_assign<'a>(
        _left: impl BergValue<'a>,
        _operator: IdentifierIndex,
        _right: RightOperand<'a, impl BergValue<'a>>,
    ) -> BergResult<'a> {
        BergError::AssignmentTargetMustBeIdentifier.err()
    }

    pub fn default_postfix<'a>(
        operand: impl BergValue<'a>,
        operator: IdentifierIndex,
    ) -> BergResult<'a> {
        BergError::UnsupportedOperator(Box::new(operand.into_result()), Fixity::Postfix, operator).err()
    }

    pub fn default_prefix<'a>(
        operand: impl BergValue<'a>,
        operator: IdentifierIndex,
    ) -> BergResult<'a> {
        use crate::syntax::identifiers::{DOUBLE_EXCLAMATION_POINT, EXCLAMATION_POINT};
        match operator {
            DOUBLE_EXCLAMATION_POINT => operand.prefix(EXCLAMATION_POINT)?.prefix(EXCLAMATION_POINT),
            _ => {
                BergError::UnsupportedOperator(Box::new(operand.into_result()), Fixity::Prefix, operator).err()
            }
        }
    }

    pub fn default_field<'a, T: BergValue<'a>>(
        object: T,
        name: IdentifierIndex,
    ) -> BergResult<'a> {
        BergError::NoSuchPublicFieldOnValue(Box::new(object.into_result()), name).err()
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn default_set_field<'a, T: BergValue<'a>+Clone>(
        object: &mut T,
        name: IdentifierIndex,
        _value: BergResult<'a>,
    ) -> BergResult<'a, ()> {
        BergError::NoSuchPublicFieldOnValue(Box::new(object.clone().into_result()), name).err()
    }
}
