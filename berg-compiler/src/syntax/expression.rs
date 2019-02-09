use crate::syntax::Fixity::*;
use crate::syntax::{
    AstIndex, Ast, AstRef, ByteRange, ExpressionBoundary, Fixity,
    OperandPosition, Token,
};
use crate::value::{BergError, BergResult, TakeError};
use std::borrow::Cow;
use std::fmt;

///
/// Implements Expression navigation: left operand, right operand, open/close
/// parens, etc.
/// 
/// Expressions are *not* meant to be kept around! Holding an expression means
/// holding a reference to an AstData, so someone else must be responsible for
/// holding the AstRef.
/// 
/// The Context parameter is convenience so that our navigation methods, like inner_expression(),
/// left_operand(), etc., will carry the context along for the ride. This
/// is used for ExpressionEvaluator.
///
#[derive(Copy, Clone)]
pub struct Expression<'p, 'a: 'p, Context: Copy+Clone+fmt::Debug = ()> {
    context: Context,
    ast: &'p Ast<'a>,
    index: AstIndex,
    position: Option<OperandPosition>,
}

#[derive(Clone)]
pub struct ExpressionRef<'a> {
    pub ast: AstRef<'a>,
    pub index: AstIndex,
}

impl<'a> ExpressionRef<'a> {
    pub fn new(ast: AstRef<'a>, index: AstIndex) -> Self {
        ExpressionRef { ast, index }
    }
    pub fn expression<'p>(&'p self) -> Expression<'p, 'a> {
        Expression::basic(&self.ast, self.index)
    }
}
impl<'a> fmt::Debug for ExpressionRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.expression())
    }
}
impl<'a> fmt::Display for ExpressionRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expression())
    }
}

impl<'p, 'a: 'p, Context: Copy+Clone+fmt::Debug> fmt::Debug for Expression<'p, 'a, Context> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reconstruct_source())
    }
}

impl<'p, 'a: 'p> fmt::Display for Expression<'p, 'a, ()> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reconstruct_source())
    }
}

impl<'p, 'a: 'p> Expression<'p, 'a, ()> {
    pub fn basic(ast: &'p Ast<'a>, index: AstIndex) -> Self {
        Expression::new((), ast, index, None)
    }
}
impl<'p, 'a: 'p, Context: Copy+Clone+fmt::Debug> Expression<'p, 'a, Context> {
    pub fn new(context: Context, ast: &'p Ast<'a>, index: AstIndex, position: Option<OperandPosition>) -> Self {
        Expression { context, ast, index, position }
    }
    pub fn with_context<C: Copy+Clone+fmt::Debug>(self, context: C) -> Expression<'p, 'a, C> {
        Expression { context, ast: self.ast, index: self.index, position: self.position }
    }
    pub fn with_operand(self, index: AstIndex, position: OperandPosition) -> Self {
        Expression { context: self.context, ast: self.ast, index, position: Some(position) }
    }
    pub fn with_index(self, index: AstIndex) -> Self {
        Expression { context: self.context, ast: self.ast, index, position: None }
    }
    pub fn context(self) -> Context {
        self.context
    }
    pub fn ast(self) -> &'p Ast<'a> {
        self.ast
    }
    pub fn index(self) -> AstIndex {
        self.index
    }
    pub fn position(self) -> Option<OperandPosition> {
        self.position
    }
    pub fn range(self) -> ByteRange {
        let start = self.ast.token_ranges()[self.first_index()].start;
        let end = self.ast.token_ranges()[self.last_index()].end;
        start..end
    }

    pub fn operator(&self) -> AstIndex {
        self.index
    }

    pub fn first_index(self) -> AstIndex {
        let token = self.token();
        match token {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => self.operator() - delta,
            _ => {
                let mut left = self;
                while left.token().has_left_operand() {
                    left = left.left_expression();
                }
                left.operator()
            }
        }
    }

    pub fn last_index(self) -> AstIndex {
        let token = self.token();
        match token {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => self.operator() + delta,
            _ => {
                let mut right = self;
                while right.token().has_right_operand() {
                    right = right.right_expression();
                }
                right.operator()
            }
        }
    }

    pub fn token(&self) -> Token {
        self.ast.tokens()[self.operator()]
    }
    pub fn token_string(self) -> Cow<'p, str> {
        let token = self.token();
        token.to_string(self.ast)
    }

    pub fn open_token(&self) -> Token {
        self.ast.tokens()[self.open_operator()]
    }

    pub fn close_token(&self) -> Token {
        self.ast.tokens()[self.close_operator()]
    }

    pub fn open_operator(&self) -> AstIndex {
        match self.token() {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => self.operator() - delta,
            _ => self.operator(),
        }
    }

    pub fn close_operator(&self) -> AstIndex {
        match self.token() {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => self.operator() + delta,
            _ => self.operator(),
        }
    }

    pub fn depth(self) -> usize {
        let mut depth = 0;
        let mut expression = self;
        while expression.index != 0 {
            depth += 1;
            expression = expression.parent();
        }
        depth
    }

    pub fn boundary(self) -> ExpressionBoundary {
        match self.open_token() {
            Token::Open { boundary, .. } => boundary,
            Token::OpenBlock { index, .. } => self.ast.blocks()[index].boundary,
            _ => unreachable!(),
        }
    }

    pub fn left_expression(self) -> Self {
        use OperandPosition::*;
        // Grab the term immediately to our left.
        let position = match self.token().fixity() {
            Fixity::Close | Fixity::Infix => Left,
            _ => PostfixOperand,
        };
        let end = self.index - 1;
        let mut start = end;

        // Pass any postfixes to find the term.
        let mut has_postfix = false;
        while self.ast.tokens()[start].fixity() == Fixity::Postfix {
            start -= 1;
            has_postfix = true;
        }

        // Jump to the open token if it's a group term (parens, curlies, etc.)
        match self.ast.tokens()[start] {
            Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => {
                start -= delta;
            }
            _ => {}
        }

        // Pass any prefixes if there is no postfix or infix.
        if position == PostfixOperand || !has_postfix {
            while start > 0 && self.ast.tokens()[start - 1].fixity() == Fixity::Prefix {
                start -= 1;
            }
        }

        // Check for an infix.
        if position != PostfixOperand && start > 0 && self.ast.tokens()[start - 1].fixity() == Fixity::Infix {
            return self.with_operand(start - 1, position);
        }

        // Pick postfix if there is one.
        if has_postfix {
            return self.with_operand(end, position);
        }

        // Otherwise, it's the leftmost index (either a prefix or term).
        self.with_operand(start, position)
    }

    pub fn right_expression(self) -> Self {
        let start = self.operator() + 1;

        match self.token().fixity() {
            // If this is prefix, it cannot have postfix or infix children, so its immediate right is the child.
            Fixity::Prefix => return self.with_operand(start, OperandPosition::PrefixOperand),
            // If this is a group term, APPLY inner() and return.
            Fixity::Open => return self.inner_expression(),
            // Otherwise, it's guaranteed to be infix.
            Fixity::Infix => {}
            _ => unreachable!(),
        }

        // Check whether there is a postfix by skipping prefix and term.
        let mut end = start;
        while self.ast.tokens()[end].fixity() == Fixity::Prefix {
            end += 1;
        }
        match self.ast.tokens()[end] {
            Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => {
                end += delta;
            }
            _ => {}
        }
        let mut has_postfix = false;
        while end < self.ast.tokens().last_index() && self.ast.tokens()[end + 1].fixity() == Fixity::Postfix {
            end += 1;
            has_postfix = true;
        }

        // If there is at least one postfix, return the outermost postfix.
        if has_postfix {
            return self.with_operand(end, OperandPosition::Right);
        }

        // Otherwise, the right child is the immediate right term (or prefix).
        self.with_operand(start, OperandPosition::Right)
    }

    pub fn parent(self) -> Self {
        // Grab the next and previous expression.
        let first_index = self.first_index();
        let last_index = self.last_index();
        let next = self.with_index(last_index + 1);
        if first_index == 0 {
            assert!(next.index <= next.ast.tokens().last_index());
            return next;
        }
        let prev = next.with_index(first_index - 1);
        if last_index >= prev.ast.tokens().last_index() {
            return prev;
        }

        // prefix > postfix > left infix > right infix > open+close
        match (prev.token().fixity(), next.token().fixity()) {
            (Infix, Postfix) | (Open, Postfix) | (Open, Infix) => next,

            (Prefix, Postfix)
            | (Prefix, Infix)
            | (Prefix, Close)
            | (Infix, Infix)
            | (Infix, Close)
            | (Open, Close) => prev,

            (Postfix, _) | (Close, _) | (Term, _) | (_, Prefix) | (_, Open) | (_, Term) => {
                unreachable!()
            }
        }
    }

    pub fn operand_position(self) -> OperandPosition {
        use self::OperandPosition::*;
        let parent = self.parent();
        match parent.token().fixity() {
            Prefix | Open => PrefixOperand,
            Postfix | Close => PostfixOperand,
            Infix if self.index < parent.index => Left,
            Infix => Right,
            Term => unreachable!(),
        }
    }

    pub fn inner_expression(self) -> Self {
        self.with_index(self.close_operator()).left_expression()
    }

    pub fn child(
        self,
        position: OperandPosition,
    ) -> Expression<'p, 'a, Context> {
        use OperandPosition::*;
        match position {
            Left | PostfixOperand => self.left_expression(),
            Right | PrefixOperand => self.right_expression(),
        }
    }
    pub fn operand(
        self,
        position: OperandPosition,
    ) -> BergResult<'a, Expression<'p, 'a, Context>> where Self: Into<ExpressionRef<'a>> {
        let operand = self.child(position);
        match operand.token() {
            Token::MissingExpression => BergError::MissingExpression.take_error(self),
            _ => Ok(operand),
        }
    }

    pub fn left_operand(self) -> BergResult<'a, Self> where Self: Into<ExpressionRef<'a>> {
        self.operand(OperandPosition::Left)
    }
    pub fn right_operand(self) -> BergResult<'a, Self> where Self: Into<ExpressionRef<'a>> {
        self.operand(OperandPosition::Right)
    }
    pub fn prefix_operand(self) -> BergResult<'a, Self> where Self: Into<ExpressionRef<'a>> {
        self.operand(OperandPosition::PrefixOperand)
    }
    pub fn postfix_operand(self) -> BergResult<'a, Self> where Self: Into<ExpressionRef<'a>> {
        self.operand(OperandPosition::PostfixOperand)
    }
}
