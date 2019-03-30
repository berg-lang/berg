use crate::syntax::{
    Ast, AstIndex, ExpressionVisitor, Expression, VisitResult, Fixity, ExpressionToken, OperatorToken
};

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
                Term(token) => self.result(walker.term(token)),
                PrefixOperator(operator) => walker.prefix(operator, self.operand(Fixity::Prefix)),
                Open(error, boundary, delta) if boundary.is_block() => {
                    let block_index = self.ast.close_block_index(self.index + delta);
                    walker.block(block_index, error, self.operand(Fixity::Open))
                }
                Open(error, boundary, _) => walker.subexpression(boundary, error, self.operand(Fixity::Open)),
            }
        };

        //
        // Next, walk operators until we reach an operator that isn't part of our expression.
        // The most important effect here is that in "(a + b + c)", "(a" (which has parent_fixity
        // == Open) will pick up "+ b" and then "+ c". Keeps the stack fairly shallow by consuming
        // operators over and over.
        //
        // TODO Consider extending this to handle precedence so that we don't have to store precedence in the AST
        //
        while let Some(operator_token) = next.walk_state.take_next_operator() {
            use OperatorToken::*;
            next = match operator_token {
                InfixOperator(operator) => walker.infix(next.result, operator, false, next.walk_state.operand(Fixity::Infix)),
                InfixAssignment(operator) => walker.infix(next.result, operator, true, next.walk_state.operand(Fixity::Infix)),
                PostfixOperator(operator) => next.walk_state.result(walker.postfix(next.result, operator)),
                Close(..) | CloseBlock(..) => unreachable!(), // next.walk_state.result(next.result),
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
