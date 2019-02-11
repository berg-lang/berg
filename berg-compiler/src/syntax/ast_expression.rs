use crate::syntax::{
    Ast, AstIndex, ExpressionVisitor, Expression, VisitResult, Fixity, ExpressionToken, OperatorToken,
};
use crate::syntax::identifiers::*;


///
/// An expression in an AST.
/// 
/// ParentFixity gives information about the parent operator: either Infix,
/// Prefix or Open. This is necessary to determine how long the operand is
/// (which type of operators to consume before the parent operator can run).
///
pub struct AstExpression<'p, 'a: 'p> {
    ast: &'p Ast<'a>,
    index: AstIndex,
    parent_fixity: Fixity,
}

impl<'p, 'a: 'p> Expression for AstExpression<'p, 'a> {
    type VisitState = Self;

    fn visit<V: ExpressionVisitor>(self, walker: &V) -> VisitResult<V, Self> {
        // First, read the first term. This includes any literal, variable
        // reference, group like () or {}, and any prefix operators in front of
        // it.
        let mut next = {
            use ExpressionToken::*;
            match self.ast.expression_token(self.index) {
                IntegerLiteral(literal) => self.result(walker.integer_literal(literal)),
                FieldReference(field) => self.result(walker.field_reference(field)),
                RawIdentifier(identifier) => self.result(walker.raw_identifier(identifier)),
                MissingExpression => self.result(walker.missing_expression()),
                ErrorTerm(error, literal) => self.result(walker.error_term(error, literal)),
                RawErrorTerm(error, raw_literal) => self.result(walker.raw_error_term(error, raw_literal)),

                PrefixOperator(operator) => walker.prefix(operator, self.operand(Fixity::Prefix)),
                Open { boundary, error, .. } => walker.subexpression(boundary, error, self.operand(Fixity::Open)),
                OpenBlock { index, error, .. } => walker.block(index, error, self.operand(Fixity::Open)),
            }
        };

        // Next, walk operators until we reach an operator that isn't part of our expression.
        // The most important effect here is that in "(a + b + c)", "(a" (which has parent_fixity
        // == Open) will pick up "+ b" and then "+ c". Keeps the stack fairly shallow by consuming
        // operators over and over.
        while let Some(operator_token) = next.walk_state.take_next_operator() {
            use OperatorToken::*;
            next = match operator_token {
                InfixOperator(operator) => walker.infix(next.result, operator, false, next.walk_state.operand(Fixity::Infix)),
                InfixAssignment(operator) => walker.infix(next.result, operator, true, next.walk_state.operand(Fixity::Infix)),
                Apply => walker.infix(next.result, APPLY, false, next.walk_state.operand(Fixity::Infix)),
                NewlineSequence => walker.infix(next.result, NEWLINE, false, next.walk_state.operand(Fixity::Infix)),

                PostfixOperator(operator) => next.walk_state.result(walker.postfix(next.result, operator)),
                Close { .. } | CloseBlock { .. } => next.walk_state.result(next.result),
            };
        }
        next
    }
}


impl<'p, 'a: 'p> AstExpression<'p, 'a> {
    ///
    /// Helper to create an operand and advance index by one.
    /// 
    fn operand(self, parent_fixity: Fixity) -> AstExpression<'p, 'a> {
        AstExpression { ast: self.ast, index: self.index + 1, parent_fixity }
    }

    ///
    /// Helper to take the result and advance index by one.
    /// 
    fn result<V: ExpressionVisitor>(self, result: V::Result) -> VisitResult<V, Self> {
        VisitResult { result, walk_state: AstExpression { ast: self.ast, index: self.index + 1, parent_fixity: self.parent_fixity } }
    }

    ///
    /// Returns the next operator if it is a right child of the current expression.
    ///
    fn take_next_operator(&self) -> Option<OperatorToken> {
        if self.index < self.ast.tokens.len() {
            let operator_token = self.ast.operator_token(self.index);
            if self.parent_fixity.takes_right_child(operator_token.fixity()) {
                return Some(operator_token);
            }
        }
        None
    }
}
