use std::borrow::Cow;
use std::fmt;
use std::ops::RangeInclusive;

use super::ast::{Ast, AstIndex, OperandPosition};
use super::bytes::ByteRange;
use super::expression_formatter::{ExpressionFormatter, ExpressionTreeFormatter};
use super::source_reconstruction::SourceReconstruction;
use super::token::{ExpressionBoundary, ExpressionToken, Fixity, OperatorToken, Token};
use Fixity::*;

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
pub struct ExpressionTreeWalker<'p, Context: Copy + Clone + fmt::Debug = ()> {
    context: Context,
    expression: AstExpressionTree<'p>,
}

#[derive(Copy, Clone)]
pub struct AstExpressionTree<'p> {
    ast: &'p Ast,
    root: AstIndex,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExpressionPosition {
    Expression,
    Left,
    Right,
    LeftLeft,
    LeftRight,
    RightLeft,
    RightRight,
}

impl<'p, Context: Copy + Clone + fmt::Debug> fmt::Debug for ExpressionTreeWalker<'p, Context> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.expression)
    }
}
impl<'p> fmt::Display for ExpressionTreeWalker<'p, ()> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}

impl<'p> fmt::Debug for AstExpressionTree<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl<'p> fmt::Display for AstExpressionTree<'p> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reconstruct_source())
    }
}

impl<'p> ExpressionTreeWalker<'p, ()> {
    pub fn basic(ast: &'p Ast, index: AstIndex) -> Self {
        ExpressionTreeWalker::new((), ast, index)
    }
}
impl<'p, Context: Copy + Clone + fmt::Debug> ExpressionTreeWalker<'p, Context> {
    pub fn new(context: Context, ast: &'p Ast, root: AstIndex) -> Self {
        ExpressionTreeWalker {
            context,
            expression: AstExpressionTree::new(ast, root),
        }
    }
    pub fn format(self) -> ExpressionTreeWalker<'p, ExpressionFormatter> {
        self.expression.format()
    }
    pub fn format_tree(self) -> ExpressionTreeWalker<'p, ExpressionTreeFormatter> {
        self.expression.format_tree()
    }
    pub fn with_context<C: Copy + Clone + fmt::Debug>(
        self,
        context: C,
    ) -> ExpressionTreeWalker<'p, C> {
        ExpressionTreeWalker {
            context,
            expression: self.expression,
        }
    }
    pub fn with_expression(self, expression: AstExpressionTree<'p>) -> Self {
        ExpressionTreeWalker {
            context: self.context,
            expression,
        }
    }
    pub fn context(self) -> Context {
        self.context
    }
    pub fn ast(self) -> &'p Ast {
        self.expression.ast()
    }
    pub fn root_index(&self) -> AstIndex {
        self.expression.root_index()
    }
    pub fn byte_range(&self) -> ByteRange {
        self.expression.byte_range()
    }
    pub fn token_range(&self) -> RangeInclusive<AstIndex> {
        self.expression.token_range()
    }
    pub fn token(&self) -> Token {
        self.expression.token()
    }
    pub fn token_string(self) -> Cow<'p, str> {
        self.expression.token_string()
    }
    pub fn open_operator(&self) -> AstIndex {
        self.expression.open_operator()
    }
    pub fn close_operator(&self) -> AstIndex {
        self.expression.close_operator()
    }
    pub fn open_token(&self) -> ExpressionToken {
        self.expression.open_token()
    }
    pub fn close_token(&self) -> OperatorToken {
        self.expression.close_token()
    }
    pub fn depth(self) -> usize {
        self.expression.depth()
    }
    pub fn boundary(self) -> ExpressionBoundary {
        self.expression.boundary()
    }
    pub fn operand_position(self) -> OperandPosition {
        self.expression.operand_position()
    }

    pub fn inner_expression(self) -> Self {
        self.with_expression(self.expression.inner_expression())
    }
    pub fn left_expression(self) -> Self {
        self.with_expression(self.expression.left_expression())
    }
    pub fn right_expression(self) -> Self {
        self.with_expression(self.expression.right_expression())
    }
    pub fn prev_expression(self) -> Self {
        self.with_expression(self.expression.prev_expression())
    }
    pub fn next_expression(self) -> Self {
        self.with_expression(self.expression.next_expression())
    }
    pub fn parent_expression(self) -> Self {
        self.with_expression(self.expression.parent_expression())
    }
    pub fn child_expression(self, position: OperandPosition) -> Self {
        self.with_expression(self.expression.child_expression(position))
    }
}

impl<'p> AstExpressionTree<'p> {
    pub fn new(ast: &'p Ast, root: AstIndex) -> Self {
        AstExpressionTree { ast, root }
    }
    pub fn ast(self) -> &'p Ast {
        self.ast
    }
    pub fn root_index(&self) -> AstIndex {
        self.root
    }
    pub fn byte_range(&self) -> ByteRange {
        let range = self.token_range();
        let start = self.ast.token_ranges[*range.start()].start;
        let end = self.ast.token_ranges[*range.end()].end;
        start..end
    }
    pub fn token_range(&self) -> RangeInclusive<AstIndex> {
        first_index(self.ast, self.root)..=last_index(self.ast, self.root)
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
    pub fn open_token(&self) -> ExpressionToken {
        self.ast
            .expression_token(open_operator_index(self.ast, self.root))
    }
    pub fn close_token(&self) -> OperatorToken {
        self.ast
            .operator_token(close_operator_index(self.ast, self.root))
    }

    pub fn depth(self) -> usize {
        let mut depth = 0;
        let mut expression = self;
        while expression.root != 0 {
            depth += 1;
            expression = expression.parent_expression();
        }
        depth
    }

    pub fn boundary(self) -> ExpressionBoundary {
        match self.open_token() {
            ExpressionToken::Open(_, boundary, _) => boundary,
            _ => unreachable!(),
        }
    }

    pub fn left_expression(self) -> Self {
        AstExpressionTree::new(self.ast, left_operand_root(self.ast, self.root))
    }

    pub fn right_expression(self) -> Self {
        AstExpressionTree::new(self.ast, right_operand_root(self.ast, self.root))
    }

    pub fn parent_expression(self) -> Self {
        AstExpressionTree::new(self.ast, parent_root(self.ast, self.root))
    }

    pub fn prev_expression(self) -> Self {
        AstExpressionTree::new(self.ast, prev_index(self.ast, self.root))
    }

    pub fn next_expression(self) -> Self {
        AstExpressionTree::new(self.ast, self.root + 1)
    }

    pub fn operand_position(self) -> OperandPosition {
        use self::OperandPosition::*;
        let parent = self.parent_expression();
        match parent.token().fixity() {
            Prefix | Open => PrefixOperand,
            Postfix | Close => PostfixOperand,
            Infix if self.root < parent.root => Left,
            Infix => Right,
            Term => unreachable!(),
        }
    }

    pub fn inner_expression(self) -> Self {
        AstExpressionTree::new(self.ast, inner_root(self.ast, self.root))
    }

    pub fn child_expression(self, position: OperandPosition) -> Self {
        use OperandPosition::*;
        match position {
            Left | PostfixOperand => self.left_expression(),
            Right | PrefixOperand => self.right_expression(),
        }
    }

    fn skip_implicit_groups(self) -> Self {
        let mut result = self;
        while let Token::Expression(ExpressionToken::Open(_, boundary, _)) = result.token() {
            if boundary.is_required() {
                break;
            }
            result = result.inner_expression();
        }
        result
    }

    pub fn at_position(self, position: ExpressionPosition) -> Self {
        use ExpressionPosition::*;
        let expression = self.skip_implicit_groups();
        let result = match position {
            Expression => expression,
            Left => expression.left_expression(),
            LeftLeft => expression
                .left_expression()
                .skip_implicit_groups()
                .left_expression(),
            LeftRight => expression
                .left_expression()
                .skip_implicit_groups()
                .right_expression(),
            Right => expression.right_expression(),
            RightLeft => expression
                .right_expression()
                .skip_implicit_groups()
                .left_expression(),
            RightRight => expression
                .right_expression()
                .skip_implicit_groups()
                .right_expression(),
        };
        result.skip_implicit_groups()
    }

    pub fn format(self) -> ExpressionTreeWalker<'p, ExpressionFormatter> {
        ExpressionTreeWalker::new(ExpressionFormatter, self.ast(), self.root_index())
    }
    pub fn format_tree(self) -> ExpressionTreeWalker<'p, ExpressionTreeFormatter> {
        ExpressionTreeWalker::new(
            ExpressionTreeFormatter {
                starting_depth: self.depth(),
            },
            self.ast(),
            self.root_index(),
        )
    }
    pub fn reconstruct_source(self) -> SourceReconstruction<'p> {
        SourceReconstruction::new(self.ast(), self.byte_range())
    }
}

///
/// The index of the token at the very beginning of the expression.
///
fn first_index(ast: &Ast, root: AstIndex) -> AstIndex {
    let token = ast.tokens[root];
    match token {
        Token::Operator(OperatorToken::Close(delta, _)) => root - delta,
        Token::Operator(OperatorToken::CloseBlock(block_index, _)) => {
            root - ast.blocks[block_index].delta
        }
        _ => {
            let mut left = root;
            while ast.tokens[left].has_left_operand() {
                left = left_operand_root(ast, left);
            }
            left
        }
    }
}

fn prev_index(ast: &Ast, root: AstIndex) -> AstIndex {
    open_operator_index(ast, root - 1)
}

///
/// The index of the token at the very end of the expression.
///
fn last_index(ast: &Ast, root: AstIndex) -> AstIndex {
    let token = ast.tokens[root];
    match token {
        Token::Expression(ExpressionToken::Open(_, _, delta)) => root + delta,
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
    use ExpressionToken::*;
    use OperatorToken::*;
    use Token::*;

    let start = root + 1;

    match ast.tokens[root] {
        // If this is prefix, it cannot have postfix or infix children, so its immediate right is the child.
        Expression(PrefixOperator(_)) => return start,
        // If this is a group term, FOLLOWED_BY inner() and return.
        Expression(Open(..)) => return inner_root(ast, root),
        // Otherwise, it's guaranteed to be infix.
        _ => assert!(ast.tokens[root].fixity() == Fixity::Infix),
    }

    // Check whether there is a postfix by skipping prefix and term.
    let mut end = start;
    while let PrefixOperator(_) = ast.expression_token(end) {
        end += 1;
    }
    if let Open(_, _, delta) = ast.expression_token(end) {
        end += delta;
    }
    let mut has_postfix = false;
    while let PostfixOperator(_) = ast.operator_token(end + 1) {
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
    use OperatorToken::*;
    use Token::*;

    let end = root - 1;
    let mut start = end;
    let is_postfix = ast.token(root).fixity() == Fixity::Postfix;

    // Pass any postfixes to find the term.
    let mut left_has_postfix = false;
    while let Operator(PostfixOperator(_)) = ast.token(start) {
        start -= 1;
        left_has_postfix = true;
    }

    // Jump to the open token if it's a group term (parens, curlies, etc.)
    match ast.token(start) {
        Operator(Close(delta, _)) => {
            start -= delta;
        }
        Operator(CloseBlock(block_index, _)) => {
            start -= ast.blocks[block_index].delta;
        }
        _ => {}
    }

    // Pass any prefixes if there is no postfix or infix.
    if is_postfix || !left_has_postfix {
        while start > 0 && ast.tokens[start - 1].fixity() == Fixity::Prefix {
            start -= 1;
        }
    }

    // Check for an infix.
    if !is_postfix && start > 0 && ast.tokens[start - 1].fixity() == Infix {
        return start - 1;
    }

    // Pick postfix if there is one.
    if left_has_postfix {
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
        Token::Operator(OperatorToken::Close(delta, _)) => index - delta,
        Token::Operator(OperatorToken::CloseBlock(block_index, _)) => {
            index - ast.blocks[block_index].delta
        }
        _ => index,
    }
}

fn close_operator_index(ast: &Ast, index: AstIndex) -> AstIndex {
    match ast.tokens[index] {
        Token::Expression(ExpressionToken::Open(_, _, delta)) => index + delta,
        _ => index,
    }
}

impl ExpressionPosition {
    pub fn relative_to(self, new_position: ExpressionPosition) -> ExpressionPosition {
        use ExpressionPosition::*;
        match (new_position, self) {
            (new_position, Expression) => new_position,
            (Expression, position) => position,
            (Left, Left) => LeftLeft,
            (Left, Right) => LeftRight,
            (Right, Left) => RightLeft,
            (Right, Right) => RightRight,
            (LeftLeft, _)
            | (LeftRight, _)
            | (RightLeft, _)
            | (RightRight, _)
            | (_, LeftLeft)
            | (_, LeftRight)
            | (_, RightLeft)
            | (_, RightRight) => unreachable!(
                "Cannot reposition {:?} on top of {:?}: too deep!",
                self, new_position
            ),
        }
    }
}
