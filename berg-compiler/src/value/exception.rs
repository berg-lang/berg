use crate::syntax::{
    ByteRange, ErrorLocation, ExpressionErrorPosition, ExpressionRef, ExpressionTreeWalker,
    IdentifierIndex, LineColumnRange,
};
use crate::value::implement::*;
use std::fmt;

///
/// Standard berg error, either with or without a full error location.
///
#[derive(Debug, Clone)]
pub enum EvalException<'a> {
    Thrown(BergVal<'a>, ExpressionErrorPosition),
    Error(Exception<'a>),
}

///
/// Thrown exception.
///
/// Includes the exception location.
///
#[derive(Debug, Clone)]
pub struct Exception<'a> {
    pub value: BergVal<'a>,
    pub expression: ExpressionRef<'a>,
}

///
/// Caught exception.
///
#[derive(Debug, Clone)]
pub struct CaughtException<'a>(Box<Exception<'a>>);

impl<'a> EvalException<'a> {
    pub fn reposition(self, new_position: ExpressionErrorPosition) -> EvalException<'a> {
        use EvalException::*;
        match self {
            Error(_) => self,
            Thrown(error, position) => Thrown(error, position.relative_to(new_position)),
        }
    }

    pub fn at_location(self, location: impl Into<ExpressionRef<'a>>) -> Exception<'a> {
        use EvalException::*;
        match self {
            Error(e) => e,
            Thrown(error, position) => Exception::new(error, location.into().at_position(position)),
        }
    }

    pub fn code(&self) -> Option<CompilerErrorCode> {
        use EvalException::*;
        match self {
            Thrown(BergVal::CompilerError(error), _) => Some(error.code()),
            Thrown(..) => None,
            Error(e) => e.code(),
        }
    }
}
impl<'a> ErrorLocation<'a> {
    pub fn range(&self) -> LineColumnRange {
        match self {
            ErrorLocation::SourceExpression(ast, _) | ErrorLocation::SourceRange(ast, _) => {
                ast.char_data.range(&self.byte_range())
            }
            _ => unreachable!(),
        }
    }
    pub fn byte_range(&self) -> ByteRange {
        match self {
            ErrorLocation::SourceExpression(ast, index) => {
                ExpressionTreeWalker::new((), ast, *index).byte_range()
            }
            ErrorLocation::SourceRange(_, range) => range.clone(),
            _ => unreachable!(),
        }
    }
}

impl<'a> Exception<'a> {
    pub fn new(value: BergVal<'a>, expression: ExpressionRef<'a>) -> Self {
        Exception { value, expression }
    }

    pub fn expression(&self) -> ExpressionRef<'a> {
        self.expression.clone()
    }

    pub fn location(&self) -> ErrorLocation<'a> {
        match self.value {
            BergVal::CompilerError(ref error) => error.location(self.expression()),
            _ => ErrorLocation::SourceExpression(self.expression.ast.clone(), self.expression.root),
        }
    }

    pub fn code(&self) -> Option<CompilerErrorCode> {
        if let BergVal::CompilerError(ref error) = self.value {
            Some(error.code())
        } else {
            None
        }
    }

    pub fn catch(self) -> CaughtException<'a> {
        CaughtException(Box::new(self))
    }
}

impl<'a> From<Exception<'a>> for EvalException<'a> {
    fn from(from: Exception<'a>) -> Self {
        EvalException::Error(from)
    }
}

impl<'a> fmt::Display for Exception<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            BergVal::CompilerError(ref error) => error.fmt_display(&self.expression, f),
            _ => write!(f, "{}", self.value),
        }
    }
}

impl<'a> Value<'a> for EvalException<'a> {
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        use EvalException::*;
        match self {
            Error(value) => value.into_native(),
            Thrown(..) => self.err(),
        }
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        use EvalException::*;
        match self {
            Error(value) => value.try_into_native(),
            Thrown(..) => self.err(),
        }
    }

    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.lazy_val(),
            Thrown(..) => self.err(),
        }
    }

    fn eval_val(self) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.eval_val(),
            Thrown(..) => self.err(),
        }
    }

    fn display(&self) -> &dyn std::fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for EvalException<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        use EvalException::*;
        match self {
            Error(value) => value.next_val(),
            Thrown(..) => self.err(),
        }
    }
}

impl<'a> ObjectValue<'a> for EvalException<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.field(name),
            Thrown(..) => self.err(),
        }
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        field_value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>>
    where
        Self: Clone,
    {
        use EvalException::*;
        match self {
            Error(ref mut value) => value.set_field(name, field_value),
            Thrown(..) => self.clone().err(),
        }
    }
}

impl<'a> OperableValue<'a> for EvalException<'a> {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.infix(operator, right),
            Thrown(error, position) => Thrown(error, position.relative_to(Left)).err(),
        }
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.infix_assign(operator, right),
            Thrown(error, position) => Thrown(error, position.relative_to(Left)).err(),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.postfix(operator),
            Thrown(error, position) => Thrown(error, position.relative_to(Left)).err(),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.prefix(operator),
            Thrown(error, position) => Thrown(error, position.relative_to(Right)).err(),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.subexpression_result(boundary),
            Thrown(error, position) => Thrown(error, position.relative_to(Right)).err(),
        }
    }
}

impl<'a> BergValue<'a> for Exception<'a> {}

impl<'a> EvaluatableValue<'a> for Exception<'a> {
    fn evaluate(self) -> BergResult<'a>
    where
        Self: Sized,
    {
        self.err()
    }
}

impl<'a> Value<'a> for Exception<'a> {
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        self.err()
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        self.err()
    }

    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>>
    where
        Self: Sized,
    {
        self.err()
    }

    fn eval_val(self) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.err()
    }

    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for Exception<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        self.err()
    }
}

impl<'a> ObjectValue<'a> for Exception<'a> {
    fn field(self, _name: IdentifierIndex) -> EvalResult<'a> {
        self.err()
    }

    fn set_field(
        &mut self,
        _name: IdentifierIndex,
        _value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>>
    where
        Self: Clone,
    {
        self.clone().err()
    }
}

impl<'a> OperableValue<'a> for Exception<'a> {
    fn infix(
        self,
        _operator: IdentifierIndex,
        _right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a> {
        self.err()
    }

    fn infix_assign(
        self,
        _operator: IdentifierIndex,
        _right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a> {
        self.err()
    }

    fn postfix(self, _operator: IdentifierIndex) -> EvalResult<'a> {
        self.err()
    }

    fn prefix(self, _operator: IdentifierIndex) -> EvalResult<'a> {
        self.err()
    }

    fn subexpression_result(self, _boundary: ExpressionBoundary) -> EvalResult<'a> {
        self.err()
    }
}

impl<'a> BergValue<'a> for CaughtException<'a> {}

impl<'a> EvaluatableValue<'a> for CaughtException<'a> {
    fn evaluate(self) -> BergResult<'a>
    where
        Self: Sized,
    {
        self.0.value.evaluate()
    }
}

impl<'a> Value<'a> for CaughtException<'a> {
    fn into_native<T: TryFromBergVal<'a>>(self) -> Result<T, EvalException<'a>> {
        self.0.value.into_native()
    }

    fn try_into_native<T: TryFromBergVal<'a>>(self) -> Result<Option<T>, EvalException<'a>> {
        self.0.value.try_into_native()
    }

    fn lazy_val(self) -> Result<BergVal<'a>, EvalException<'a>>
    where
        Self: Sized,
    {
        self.ok()
    }

    fn eval_val(self) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.ok()
    }

    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl<'a> IteratorValue<'a> for CaughtException<'a> {
    fn next_val(self) -> Result<NextVal<'a>, EvalException<'a>> {
        self.0.value.next_val()
    }
}

impl<'a> ObjectValue<'a> for CaughtException<'a> {
    fn field(self, name: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.0.value.field(name)
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal<'a>,
    ) -> Result<(), EvalException<'a>>
    where
        Self: Clone,
    {
        self.0.value.set_field(name, value)
    }
}

impl<'a> OperableValue<'a> for CaughtException<'a> {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.0.value.infix(operator, right)
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<'a, impl EvaluatableValue<'a>>,
    ) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.0.value.infix_assign(operator, right)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.0.value.postfix(operator)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.0.value.prefix(operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult<'a>
    where
        Self: Sized,
    {
        self.0.value.subexpression_result(boundary)
    }
}

impl<'a> TryFromBergVal<'a> for CaughtException<'a> {
    const TYPE_NAME: &'static str = "CaughtException";
    fn try_from_berg_val(
        from: EvalVal<'a>,
    ) -> Result<Result<Self, BergVal<'a>>, EvalException<'a>> {
        match from.lazy_val()?.evaluate()? {
            BergVal::CaughtException(value) => Ok(Ok(value)),
            from => Ok(Err(from)),
        }
    }
}

impl<'a> From<CaughtException<'a>> for BergVal<'a> {
    fn from(from: CaughtException<'a>) -> Self {
        BergVal::CaughtException(from)
    }
}

impl<'a> From<CaughtException<'a>> for EvalVal<'a> {
    fn from(from: CaughtException<'a>) -> Self {
        BergVal::from(from).into()
    }
}

impl<'a> fmt::Display for CaughtException<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
