use crate::syntax::{
    BlockIndex, ExpressionBoundary, ExpressionBoundaryError, IdentifierIndex, LiteralIndex, RawLiteralIndex, FieldIndex,
};
use crate::value::ErrorCode;

///
/// A visitor that is passed expressions in evaluation order.
///
pub trait ExpressionVisitor: Sized {
    type Result: Sized;
    fn integer_literal(&self, literal: LiteralIndex) -> Self::Result;
    fn field_reference(&self, field: FieldIndex) -> Self::Result;
    fn raw_identifier(&self, identifier: IdentifierIndex) -> Self::Result;
    fn missing_expression(&self) -> Self::Result;
    fn error_term(&self, error: ErrorCode, literal: LiteralIndex) -> Self::Result;
    fn raw_error_term(&self, error: ErrorCode, literal: RawLiteralIndex) -> Self::Result;
    fn postfix(&self, left: Self::Result, operator: IdentifierIndex) -> Self::Result;

    fn infix<E: Expression>(&self, left: Self::Result, operator: IdentifierIndex, is_assign: bool, operand: E) -> VisitResult<Self, E>;
    fn prefix<E: Expression>(&self, operator: IdentifierIndex, operand: E) -> VisitResult<Self, E>;
    fn subexpression<E: Expression>(&self, _boundary: ExpressionBoundary, _error: ExpressionBoundaryError, inner: E) -> VisitResult<Self, E> {
        inner.visit_and(self, |r| r)
    }
    fn block<E: Expression>(&self, block: BlockIndex, error: ExpressionBoundaryError, inner: E) -> VisitResult<Self, E>;
}

pub struct VisitResult<V: ExpressionVisitor, E: Expression> {
    pub result: V::Result,
    pub walk_state: E::VisitState,
}

pub trait Expression: Sized {
    ///
    /// Internal state held by an expression. Used to construct the VisitResult type.
    ///
    type VisitState;

    ///
    /// Walk this expression.
    /// 
    fn visit<V: ExpressionVisitor>(self, walker: &V) -> VisitResult<V, Self>;

    fn visit_and<V: ExpressionVisitor, F: FnOnce(V::Result) -> V::Result>(self, walker: &V, f: F) -> VisitResult<V, Self> {
        let mut result = self.visit(walker);
        result.result = f(result.result);
        result
    }

    ///
    /// Skip this expression.
    ///
    fn skip<V: ExpressionVisitor>(self, result: V::Result) -> VisitResult<V, Self> {
        let walk_state = self.visit(&SkipExpression).walk_state;
        VisitResult { result, walk_state }
    }
}

///
/// Walker that skips an expression.
///
#[allow(dead_code)]
struct SkipExpression;

impl ExpressionVisitor for SkipExpression {
    type Result = ();
    fn integer_literal(&self, _literal: LiteralIndex) -> Self::Result { }
    fn field_reference(&self, _field: FieldIndex) -> Self::Result { }
    fn raw_identifier(&self, _identifier: IdentifierIndex) -> Self::Result { }
    fn missing_expression(&self) -> Self::Result { }
    fn error_term(&self, _error: ErrorCode, _literal: LiteralIndex) -> Self::Result { }
    fn raw_error_term(&self, _error: ErrorCode, _literal: RawLiteralIndex) -> Self::Result { }
    fn postfix(&self, _left: Self::Result, _operator: IdentifierIndex) -> Self::Result { }
    fn infix<E: Expression>(&self, _left: Self::Result, _operator: IdentifierIndex, _is_assign: bool, operand: E) -> VisitResult<Self, E> {
        operand.visit(self)
    }
    fn prefix<E: Expression>(&self, _operator: IdentifierIndex, operand: E) -> VisitResult<Self, E> {
        operand.visit(self)
    }
    fn subexpression<E: Expression>(&self, _boundary: ExpressionBoundary, _error: ExpressionBoundaryError, inner: E) -> VisitResult<Self, E> {
        inner.visit(self)
    }
    fn block<E: Expression>(&self, _block: BlockIndex, _error: ExpressionBoundaryError, inner: E) -> VisitResult<Self, E> {
        inner.visit(self)
    }
}

