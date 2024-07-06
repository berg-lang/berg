use super::implement::*;
use berg_parser::{
    AstIndex, ByteRange, ExpressionPosition, ExpressionTreeWalker, IdentifierIndex, LineColumnRange,
};
use std::fmt;

///
/// Standard berg error, either with or without a full error location.
///
#[derive(Debug, Clone)]
pub enum EvalException {
    Thrown(BergVal, ExpressionPosition),
    Error(Exception),
}

///
/// Thrown exception.
///
/// Includes the exception location.
///
#[derive(Debug, Clone)]
pub struct Exception {
    pub value: BergVal,
    pub expression: ExpressionRef,
}

///
/// Caught exception.
///
#[derive(Debug, Clone)]
pub struct CaughtException(Box<Exception>);

#[derive(Debug, Clone)]
pub enum ErrorLocation {
    Generic,
    SourceOnly(AstRef),
    SourceExpression(AstRef, AstIndex),
    SourceRange(AstRef, ByteRange),
}

impl EvalException {
    pub fn reposition(self, new_position: ExpressionPosition) -> EvalException {
        use EvalException::*;
        match self {
            Error(_) => self,
            Thrown(error, position) => Thrown(error, position.relative_to(new_position)),
        }
    }

    pub fn at_location(self, location: impl Into<ExpressionRef>) -> Exception {
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
impl ErrorLocation {
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

impl Exception {
    pub fn new(value: BergVal, expression: ExpressionRef) -> Self {
        Exception { value, expression }
    }

    pub fn expression(&self) -> ExpressionRef {
        self.expression.clone()
    }

    pub fn location(&self) -> ErrorLocation {
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

    pub fn catch(self) -> CaughtException {
        CaughtException(Box::new(self))
    }
}

impl From<Exception> for EvalException {
    fn from(from: Exception) -> Self {
        EvalException::Error(from)
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            BergVal::CompilerError(ref error) => error.fmt_display(&self.expression, f),
            _ => write!(f, "{}", self.value),
        }
    }
}

impl Value for EvalException {
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        use EvalException::*;
        match self {
            Error(value) => value.into_native(),
            Thrown(..) => self.err(),
        }
    }

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        use EvalException::*;
        match self {
            Error(value) => value.try_into_native(),
            Thrown(..) => self.err(),
        }
    }

    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.lazy_val(),
            Thrown(..) => self.err(),
        }
    }

    fn eval_val(self) -> EvalResult
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

impl IteratorValue for EvalException {
    fn next_val(self) -> Result<NextVal, EvalException> {
        use EvalException::*;
        match self {
            Error(value) => value.next_val(),
            Thrown(..) => self.err(),
        }
    }
}

impl ObjectValue for EvalException {
    fn field(self, name: IdentifierIndex) -> EvalResult
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
        field_value: BergVal,
    ) -> Result<(), EvalException>
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

impl OperableValue for EvalException {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
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
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.infix_assign(operator, right),
            Thrown(error, position) => Thrown(error, position.relative_to(Left)).err(),
        }
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.postfix(operator),
            Thrown(error, position) => Thrown(error, position.relative_to(Left)).err(),
        }
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        use EvalException::*;
        match self {
            Error(value) => value.prefix(operator),
            Thrown(error, position) => Thrown(error, position.relative_to(Right)).err(),
        }
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
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

impl BergValue for Exception {}

impl EvaluatableValue for Exception {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.err()
    }
}

impl Value for Exception {
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        self.err()
    }

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        self.err()
    }

    fn lazy_val(self) -> Result<BergVal, EvalException>
    where
        Self: Sized,
    {
        self.err()
    }

    fn eval_val(self) -> EvalResult
    where
        Self: Sized,
    {
        self.err()
    }

    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl IteratorValue for Exception {
    fn next_val(self) -> Result<NextVal, EvalException> {
        self.err()
    }
}

impl ObjectValue for Exception {
    fn field(self, _name: IdentifierIndex) -> EvalResult {
        self.err()
    }

    fn set_field(
        &mut self,
        _name: IdentifierIndex,
        _value: BergVal,
    ) -> Result<(), EvalException>
    where
        Self: Clone,
    {
        self.clone().err()
    }
}

impl OperableValue for Exception {
    fn infix(
        self,
        _operator: IdentifierIndex,
        _right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult {
        self.err()
    }

    fn infix_assign(
        self,
        _operator: IdentifierIndex,
        _right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult {
        self.err()
    }

    fn postfix(self, _operator: IdentifierIndex) -> EvalResult {
        self.err()
    }

    fn prefix(self, _operator: IdentifierIndex) -> EvalResult {
        self.err()
    }

    fn subexpression_result(self, _boundary: ExpressionBoundary) -> EvalResult {
        self.err()
    }
}

impl BergValue for CaughtException {}

impl EvaluatableValue for CaughtException {
    fn evaluate(self) -> BergResult
    where
        Self: Sized,
    {
        self.0.value.evaluate()
    }
}

impl Value for CaughtException {
    fn into_native<T: TryFromBergVal>(self) -> Result<T, EvalException> {
        self.0.value.into_native()
    }

    fn try_into_native<T: TryFromBergVal>(self) -> Result<Option<T>, EvalException> {
        self.0.value.try_into_native()
    }

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

    fn display(&self) -> &dyn fmt::Display {
        self
    }
}

impl IteratorValue for CaughtException {
    fn next_val(self) -> Result<NextVal, EvalException> {
        self.0.value.next_val()
    }
}

impl ObjectValue for CaughtException {
    fn field(self, name: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        self.0.value.field(name)
    }

    fn set_field(
        &mut self,
        name: IdentifierIndex,
        value: BergVal,
    ) -> Result<(), EvalException>
    where
        Self: Clone,
    {
        self.0.value.set_field(name, value)
    }
}

impl OperableValue for CaughtException {
    fn infix(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        self.0.value.infix(operator, right)
    }

    fn infix_assign(
        self,
        operator: IdentifierIndex,
        right: RightOperand<impl EvaluatableValue>,
    ) -> EvalResult
    where
        Self: Sized,
    {
        self.0.value.infix_assign(operator, right)
    }

    fn postfix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        self.0.value.postfix(operator)
    }

    fn prefix(self, operator: IdentifierIndex) -> EvalResult
    where
        Self: Sized,
    {
        self.0.value.prefix(operator)
    }

    fn subexpression_result(self, boundary: ExpressionBoundary) -> EvalResult
    where
        Self: Sized,
    {
        self.0.value.subexpression_result(boundary)
    }
}

impl TryFromBergVal for CaughtException {
    const TYPE_NAME: &'static str = "CaughtException";
    fn try_from_berg_val(
        from: EvalVal,
    ) -> Result<Result<Self, BergVal>, EvalException> {
        match from.lazy_val()?.evaluate()? {
            BergVal::CaughtException(value) => Ok(Ok(value)),
            from => Ok(Err(from)),
        }
    }
}

impl From<CaughtException> for BergVal {
    fn from(from: CaughtException) -> Self {
        BergVal::CaughtException(from)
    }
}

impl From<CaughtException> for EvalVal {
    fn from(from: CaughtException) -> Self {
        BergVal::from(from).into()
    }
}

impl fmt::Display for CaughtException {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
