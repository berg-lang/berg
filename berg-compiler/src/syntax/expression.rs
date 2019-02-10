use crate::syntax::Fixity::*;
use crate::syntax::{
    Ast, AstIndex, AstRef, ByteRange, ExpressionBoundary, Fixity, OperandPosition, Token,
};
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
pub struct Expression<'p, 'a: 'p, Context: Copy + Clone + fmt::Debug = ()> {
    context: Context,
    ast: &'p Ast<'a>,
    root: AstIndex,
}

#[derive(Clone)]
pub struct ExpressionRef<'a> {
    pub ast: AstRef<'a>,
    pub root: AstIndex,
}

impl<'a> ExpressionRef<'a> {
    pub fn new(ast: AstRef<'a>, root: AstIndex) -> Self {
        ExpressionRef { ast, root }
    }
    pub fn expression<'p>(&'p self) -> Expression<'p, 'a> {
        Expression::basic(&self.ast, self.root)
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

impl<'p, 'a: 'p, Context: Copy + Clone + fmt::Debug> fmt::Debug for Expression<'p, 'a, Context> {
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
        Expression::new((), ast, index)
    }
}
impl<'p, 'a: 'p, Context: Copy + Clone + fmt::Debug> Expression<'p, 'a, Context> {
    pub fn new(
        context: Context,
        ast: &'p Ast<'a>,
        root: AstIndex,
    ) -> Self {
        Expression {
            context,
            ast,
            root,
        }
    }
    pub fn with_context<C: Copy + Clone + fmt::Debug>(self, context: C) -> Expression<'p, 'a, C> {
        Expression {
            context,
            ast: self.ast,
            root: self.root,
        }
    }
    pub fn with_root(self, root: AstIndex) -> Self {
        Expression {
            context: self.context,
            ast: self.ast,
            root,
        }
    }
    pub fn context(self) -> Context {
        self.context
    }
    pub fn ast(self) -> &'p Ast<'a> {
        self.ast
    }
    pub fn index(self) -> AstIndex {
        self.root
    }
    pub fn range(self) -> ByteRange {
        let start = self.ast.token_ranges[first_index(self.ast, self.root)].start;
        let end = self.ast.token_ranges[last_index(self.ast, self.root)].end;
        start..end
    }

    pub fn token(&self) -> Token {
        self.ast.tokens[self.root]
    }
    pub fn token_string(self) -> Cow<'p, str> {
        let token = self.token();
        token.to_string(self.ast)
    }

    pub fn open_operator(&self) -> AstIndex {
        open_operator_index(self.ast, self.root)
    }
    pub fn close_operator(&self) -> AstIndex {
        close_operator_index(self.ast, self.root)
    }

    pub fn open_token(&self) -> Token {
        self.ast.tokens[open_operator_index(self.ast, self.root)]
    }

    pub fn close_token(&self) -> Token {
        self.ast.tokens[close_operator_index(self.ast, self.root)]
    }

    pub fn depth(self) -> usize {
        let mut depth = 0;
        let mut expression = self;
        while expression.root != 0 {
            depth += 1;
            expression = expression.parent();
        }
        depth
    }

    pub fn boundary(self) -> ExpressionBoundary {
        match self.open_token() {
            Token::Open { boundary, .. } => boundary,
            Token::OpenBlock { index, .. } => self.ast.blocks[index].boundary,
            _ => unreachable!(),
        }
    }

    pub fn left_expression(self) -> Self {
        self.with_root(left_operand_root(self.ast, self.root))
    }

    pub fn right_expression(self) -> Self {
        self.with_root(right_operand_root(self.ast, self.root))
    }

    pub fn parent(self) -> Self {
        self.with_root(parent_root(self.ast, self.root))
    }

    pub fn operand_position(self) -> OperandPosition {
        use self::OperandPosition::*;
        let parent = self.parent();
        match parent.token().fixity() {
            Prefix | Open => PrefixOperand,
            Postfix | Close => PostfixOperand,
            Infix if self.root < parent.root => Left,
            Infix => Right,
            Term => unreachable!(),
        }
    }

    pub fn inner_expression(self) -> Self {
        self.with_root(inner_root(self.ast, self.root))
    }

    pub fn child(self, position: OperandPosition) -> Expression<'p, 'a, Context> {
        use OperandPosition::*;
        match position {
            Left | PostfixOperand => self.left_expression(),
            Right | PrefixOperand => self.right_expression(),
        }
    }
}

///
/// The index of the token at the very end of the expression.
///
fn first_index(ast: &Ast, root: AstIndex) -> AstIndex {
    let token = ast.tokens[root];
    match token {
        Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => root - delta,
        _ => {
            let mut left = root;
            while ast.tokens[left].has_left_operand() {
                left = left_operand_root(ast, left);
            }
            left
        }
    }
}

///
/// The index of the token at the very end of the expression.
///
fn last_index(ast: &Ast, root: AstIndex) -> AstIndex {
    let token = ast.tokens[root];
    match token {
        Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => root + delta,
        _ => {
            let mut right = root;
            while ast.tokens[right].has_right_operand() {
                right = right_operand_root(ast, right);
            }
            right
        }
    }
}

///
/// The root index of the current expression's right operand.
///
fn right_operand_root(ast: &Ast, root: AstIndex) -> AstIndex {
    use Fixity::*;
    let start = root + 1;

    match ast.tokens[root].fixity() {
        // If this is prefix, it cannot have postfix or infix children, so its immediate right is the child.
        Prefix => return start,
        // If this is a group term, APPLY inner() and return.
        Open => return inner_root(ast, root),
        // Otherwise, it's guaranteed to be infix.
        Infix => {}
        _ => unreachable!(),
    }

    // Check whether there is a postfix by skipping prefix and term.
    let mut end = start;
    while ast.tokens[end].fixity() == Fixity::Prefix {
        end += 1;
    }
    match ast.tokens[end] {
        Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => {
            end += delta;
        }
        _ => {}
    }
    let mut has_postfix = false;
    while end < ast.tokens.last_index()
        && ast.tokens[end + 1].fixity() == Fixity::Postfix
    {
        end += 1;
        has_postfix = true;
    }

    // If there is at least one postfix, return the outermost postfix.
    if has_postfix {
        return end;
    }

    // Otherwise, the right child is the immediate right term (or prefix).
    start
}

fn left_operand_root(ast: &Ast, root: AstIndex) -> AstIndex {
    use Fixity::*;
    let end = root - 1;
    let mut start = end;

    // Pass any postfixes to find the term.
    let mut has_postfix = false;
    while ast.tokens[start].fixity() == Fixity::Postfix {
        start -= 1;
        has_postfix = true;
    }

    // Jump to the open token if it's a group term (parens, curlies, etc.)
    match ast.tokens[start] {
        Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => {
            start -= delta;
        }
        _ => {}
    }

    // Pass any prefixes if there is no postfix or infix.
    if ast.tokens[root].fixity() == Postfix || !has_postfix {
        while start > 0 && ast.tokens[start - 1].fixity() == Fixity::Prefix {
            start -= 1;
        }
    }

    // Check for an infix.
    if ast.tokens[root].fixity() != Postfix
        && start > 0
        && ast.tokens[start - 1].fixity() == Infix
    {
        return start - 1;
    }

    // Pick postfix if there is one.
    if has_postfix {
        return end;
    }

    // Otherwise, it's the leftmost index (either a prefix or term).
    start
}

fn parent_root(ast: &Ast, root: AstIndex) -> AstIndex {
    // Grab the next and previous expression.
    let first_index = first_index(ast, root);
    let last_index = last_index(ast, root);
    let next = last_index + 1;
    if first_index == 0 {
        assert!(next <= ast.tokens.last_index());
        return next;
    }
    let prev = first_index - 1;
    if last_index >= ast.tokens.last_index() {
        return prev;
    }

    // prefix > postfix > left infix > right infix > open+close
    match (ast.tokens[prev].fixity(), ast.tokens[next].fixity()) {
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

fn inner_root(ast: &Ast, index: AstIndex) -> AstIndex {
    let close = close_operator_index(ast, index);
    left_operand_root(ast, close)
}

fn open_operator_index(ast: &Ast, index: AstIndex) -> AstIndex {
    match ast.tokens[index] {
        Token::Close { delta, .. } | Token::CloseBlock { delta, .. } => index - delta,
        _ => index,
    }
}

fn close_operator_index(ast: &Ast, index: AstIndex) -> AstIndex {
    match ast.tokens[index] {
        Token::Open { delta, .. } | Token::OpenBlock { delta, .. } => index + delta,
        _ => index,
    }
}

