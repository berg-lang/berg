use crate::parser::Binder;
use crate::syntax::ExpressionBoundary::*;
use crate::syntax::ExpressionBoundaryError::*;
use crate::syntax::ExpressionToken::*;
use crate::syntax::OperatorToken::*;
use crate::syntax::TermToken::*;
use crate::syntax::{
    Ast, AstDelta, AstIndex, ByteIndex, ByteRange, ExpressionBoundary, ExpressionBoundaryError, Token, OperatorToken, ExpressionToken
};
use crate::util::indexed_vec::Delta;

// Handles nesting and precedence: balances (), {}, and compound terms, and
// inserts "precedence groups," and removes compound terms and precedence
// groups where it can.
#[derive(Debug)]
pub struct Grouper<'a> {
    binder: Binder<'a>,
    open_expressions: Vec<OpenExpression>,
//    current_line_indent: Delta<ByteIndex>,
    start_auto_block: bool,
}

// ///
// /// Represents an unclosed expression (parentheses, block, precedence, etc.).
// /// 
// #[derive(Debug)]
// enum OpenExpression {
//     ///
//     /// An explicit grouping.
//     ///
//     /// For example the `(` in `(1 + 2)` and then `{` in `{ :x = 1; x }`
//     /// 
//     /// [`AstIndex`] points to the token that opened the group.
//     /// [`Delta<ByteIndex>`] is the indent of the line the open token was on.
//     /// 
//     /// Any line *less* indented or with the *same* indent as the group start
//     /// will end it with an "open without close" error.
//     /// 
//     Explicit(AstIndex, ExpressionBoundary, Delta<ByteIndex>),

//     ///
//     /// A precedence group.
//     /// 
//     /// For example, in `a + b * c`, when we see `+`, we open a precedence group
//     /// for `(b * c)` and store the `+` so we can check whether future operators
//     /// are or are not part of the precedence group.
//     /// 
//     /// [`AstIndex`] points to the `OperatorToken` that started this group.
//     /// 
//     Precedence(AstIndex),

//     ///
//     /// A compound term (expression with no whitespace).
//     /// 
//     /// For example, `1+2` or `3+4` in `1+2 * 3+4`.
//     /// [`AstIndex`] points to the `ExpressionToken` that started this group.
//     /// 
//     Term(AstIndex),

//     ///
//     /// A possible AutoBlock.
//     /// 
//     /// This is used when an AutoBlock operator has been seen and we don't yet
//     /// know whether the expression will be on the *same* line, or the *next*
//     /// line.
//     /// 
//     /// For example: the `XPlusOneSquared:` in `XPlusOneSquared: (X+1)*(X+1)` or
//     /// 
//     ///     XPlusOneSquared:
//     ///         :X1 = X+1
//     ///         X1*X1
//     /// 
//     /// [`AstIndex`] points to the operator that started the auto block (e.g. `:`).
//     /// 
//     /// This will change to Precedence or AutoBlock depending on whether a token
//     /// is found on the next line.
//     /// 
//     MaybeAutoBlock(AstIndex),

//     ///
//     /// An auto block with indent.
//     /// 
//     /// For example:
//     /// 
//     ///     OriginPoint:
//     ///         X: 0
//     ///         Y: 0
//     /// 
//     /// It is possible for the first line to be non-empty and it still be a block.
//     /// To wit:
//     /// 
//     ///     PrintNumbers: while x < 10
//     ///         print x
//     ///         x++
//     /// 
//     /// [`AstIndex`] points to the OpenBlock token that was inserted after the
//     /// operator. `Delta<ByteIndex>` is the indent of the line the `:` was on.
//     /// All lines more indented than that are part of the block.
//     /// 
//     AutoBlock(AstIndex, Delta<ByteIndex>),
// }

///
/// An open expression group.
/// 
/// Represents parentheses, blocks, precedence, terms.
/// 
#[derive(Debug)]
struct OpenExpression {
    ///
    /// The infix operator that opened this group (if it is a precedence expression).
    /// 
    /// For example, in `a + b * c`, when we see `+`, we open a precedence group
    /// for `(b * c)` and store the `+` so we can check whether future operators
    /// are or are not part of the precedence group.
    /// 
    infix: Option<(OperatorToken, AstIndex)>,
    ///
    /// The index of the open token for this expression.
    /// 
    /// If [`boundary.is_required()`] is false, the token hasn't been added yet
    /// so this is the *intended insertion point*. If it's true, it's the index
    /// of the actual token.
    /// 
    open_index: AstIndex,
    ///
    /// The type of expression group.
    /// 
    boundary: ExpressionBoundary,
    // ///
    // /// The indent level of the current block. If `None`, the block has not yet
    // /// had an indented line.
    // ///
    // indent: Option<Delta<ByteIndex>>,
}

impl<'a> Grouper<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        Grouper {
            binder: Binder::new(ast),
            open_expressions: Default::default(),
            start_auto_block: false,
        }
    }

    pub fn ast(&self) -> &Ast<'a> {
        &self.binder.ast
    }

    pub fn ast_mut(&mut self) -> &mut Ast<'a> {
        &mut self.binder.ast
    }

    pub fn on_expression_token(&mut self, token: ExpressionToken, range: ByteRange) {
        // If we need to start an auto block, do so at this point! This allows the block to start
        // at the actual start of the block's expression.
        if self.start_auto_block {
            self.start_auto_block = false;
            // If we're supposed to start an auto block, but don't have an expression,
            // just skip the auto block.
            if token != MissingExpression.into() {
                self.on_expression_token(Open(None, AutoBlock, 0.into()), range.start..range.start);
            }
        }
        match token {
            Term(_) | PrefixOperator(_) => { self.push_expression_token(token, range); },
            Open(error, boundary, ..) => self.on_open_token(boundary, error, range)
        }
    }

    pub fn on_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        match token {
            // Delay the close token so that we can see the next infix.
            Close(_, boundary) => self.on_close_token(boundary, range),

            // Infix tokens may have left->right or right->left precedence.
            InfixOperator(_) | NewlineSequence(_) | InfixAssignment(_) => {
                // Close parent groups that don't want us as a child.
                while !self.open_expression_wants_child(token) {
                    self.close_top(None, range.start..range.start);
                }

                // Open a precedence group if it's needed.
                self.open_precedence_group_if_needed(token);

                // Add the infix token, lest we forget!
                let index = self.push_operator_token(token, range);

                // Set this as the last infix of the current open expression.
                self.open_expressions.last_mut().unwrap().infix = Some((token, index));

                if token.starts_auto_block() {
                    self.start_auto_block = true;
                }
            }

            PostfixOperator(_) => { self.push_operator_token(token, range); }
            CloseBlock(..) => unreachable!(),
        }
    }

    pub fn on_source_end(self) -> Ast<'a> {
        self.binder.on_source_end()
    }

    fn open_precedence_group_if_needed(&mut self, next_infix: OperatorToken) {
        // Close any parent precedence groups unless they want this infix as a child.
        // If we are a right child of the parent, we need to wrap ourselves
        // in an "invisible parentheses" (a precedence subexpression).
        // e.g. 1+2*3 -> 1+2 -> 1+(2* ...
        // e.g. 1*2>3+4 -> 1*2>(3+ ...
        // e.g. 1+2>3*4 -> 1+2>(3* ...
        // e.g. 1>2+3*4 -> 1>(2+(3* ...
        let open_expression = self.open_expression();
        if let Some((infix, index)) = open_expression.infix {
            if infix.takes_right_child(next_infix) {
                self.open_expressions.push(OpenExpression {
                    open_index: index + 1,
                    boundary: PrecedenceGroup,
                    infix: None,
                    // indent: open_expression.indent
                });
            }
        }
    }

    fn open_expression_wants_child(&self, next_infix: impl Into<Token>) -> bool {
        use crate::syntax::ExpressionBoundary::*;
        let infix = match self.open_expression().boundary {
            // The autoblock wants whatever its *parent* infix wants.
            AutoBlock => {
                println!("{:?}", self.open_expressions[self.open_expressions.len() - 2]);
                self.open_expressions[self.open_expressions.len() - 2].infix
            },
            PrecedenceGroup => self.open_expression().infix,
            _ => return true,
        };
        if let Some((infix, _)) = infix {
            infix.takes_right_child(next_infix)
        } else {
            true
        }
    }

    fn open_expression(&self) -> &OpenExpression {
        self.open_expressions.last().unwrap()
    }

    // fn on_indent(&mut self, indent: ByteRange) {
    //     self.open_block()
    // }

    fn on_close_token(
        &mut self,
        boundary: ExpressionBoundary,
        range: ByteRange,
    ) {
        // We never get PrecedenceGroup close tokens. Using > here in case another
        // boundary type is inserted with lower precedence than PrecedenceGroup,
        // to trigger an error and force this to be rethought.
        assert!(boundary > ExpressionBoundary::PrecedenceGroup);

        loop {
            use std::cmp::Ordering::*;
            let open_boundary = self.open_expression().boundary;
            match boundary.partial_cmp(&open_boundary).unwrap() {
                // If we are HIGHER priority than the current expression (e.g. "( ... }"), the top
                // expression must be closed even though it is unmatched.
                Greater => {
                    let error = if open_boundary.is_closed_automatically() {
                        None
                    } else {
                        Some(OpenWithoutClose)
                    };
                    self.close_top(error, range.start..range.start);
                }
                // If they are the same, then we treat them as the same exact pair. This closes
                // the boundary.
                Equal => {
                    self.close_top(None, range);
                    break;
                }
                // If we are LOWER priority than the current expression (e.g. "{ ... )"), the close
                // token is unmatched and will be opened right after the open of the parent expression
                // and ). Insert it, close this token, and we're done.
                Less => {
                    // We make the assumption here that the open token has already been emitted here,
                    // which only happens when open_boundary.is_required(). This assumption is
                    // presently true because compound terms and precedence groups are the only
                    // optional boundaries, and due to the assert from above we will never be called
                    // with a precedence group boundary.
                    assert!(open_boundary.is_required());

                    let open_index = self.open_expression().open_index + 1;
                    let error = if boundary.is_required() { Some(CloseWithoutOpen) } else { None };
                    self.close(open_index, boundary, error, range);
                    break;
                }
            }
        }
    }

    fn close_top(&mut self, error: Option<ExpressionBoundaryError>, range: ByteRange) {
        if let Some(expression) = self.pop_open_expression() {
            self.close(expression.open_index, expression.boundary, error, range);
        }
    }

    fn close(&mut self, open_index: AstIndex, boundary: ExpressionBoundary, error: Option<ExpressionBoundaryError>, range: ByteRange) {
        if boundary.is_required() && error != Some(CloseWithoutOpen) {
            self.push_close_token(open_index, boundary, error, range);
        } else {
            // If it's not required, or if this is a close without open, we never inserted the open
            // token in the first place. Fix it by inserting the open token and then pushing the close token.
            self.insert_token_pair(open_index, boundary, error, range);
        }
    }

    fn push_open_expression(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        infix: Option<(OperatorToken, AstIndex)>,
    ) {
        let open_expression = OpenExpression {
            open_index,
            boundary,
            infix,
        };
        self.open_expressions.push(open_expression);
    }

    fn insert_open_token(&mut self, index: AstIndex, error: Option<ExpressionBoundaryError>, boundary: ExpressionBoundary, delta: AstDelta, range: ByteRange) {
        self.binder.insert_open_token(index, error, boundary, delta, range)
    }

    fn push_expression_token(&mut self, token: ExpressionToken, range: ByteRange) -> AstIndex {
        self.binder.push_expression_token(token, range)
    }

    fn push_operator_token(&mut self, token: OperatorToken, range: ByteRange) -> AstIndex {
        self.binder.push_operator_token(token, range)
    }

    fn push_open_token(
        &mut self,
        boundary: ExpressionBoundary,
        error: Option<ExpressionBoundaryError>,
        open_range: ByteRange,
    ) -> AstIndex {
        let open_token = boundary.placeholder_open_token(error);
        self.push_expression_token(open_token, open_range)
    }

    fn push_close_token(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        error: Option<ExpressionBoundaryError>,
        close_range: ByteRange,
    ) -> AstIndex {
        let close_index = self.ast().next_index();
        let delta = close_index - open_index;

        // Update open index and add error
        {
            let ast = self.ast_mut();
            match ast.tokens[open_index] {
                Token::Expression(Open(ref mut open_error, open_boundary, ref mut open_delta)) => {
                    assert_eq!(open_boundary, boundary);
                    *open_delta = delta;
                    if error.is_some() {
                        // OpenWithoutClose and CloseWithoutOpen cannot hit the same paren pair.
                        // OpenError only applies to the top level, which cannot have a missing close or open, either.
                        assert_eq!(*open_error, None);
                        *open_error = error;
                    }
                }
                _ => unreachable!("{}: {:?}", open_index, ast.tokens[open_index]),
            }
        }

        let close_token = Close(delta, boundary);
        let index = self.push_operator_token(close_token, close_range);
        assert_eq!(close_index, index);
        index
    }

    ///
    /// Insert both an open and close token.
    /// 
    /// Used for groups like precedence groups, which are elided so often that
    /// we don't bother inserting the open token when we first see them. We
    /// then *insert* the token if we *actually* need the precedence group to
    /// resolve an ambiguity.
    /// 
    /// Also used when we see an unmatched close token like ) or }: we insert
    /// the open token at the beginning of the current group, as if the user had
    /// typed it there. e.g. {1 + 2)} emits an error but guesses you meant to
    /// type {(1 + 2)} rather than {1 + 2()}.
    /// 
    fn insert_token_pair(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        error: Option<ExpressionBoundaryError>,
        close_range: ByteRange,
    ) -> AstIndex {
        let open_start = self.ast().token_ranges[open_index].start;
        let close_index = self.ast().next_index() + 1; // Have to add 1 due to the impending insert.
        let delta = close_index - open_index;
        let close_token = Close(delta, boundary);
        self.insert_open_token(open_index, error, boundary, delta, open_start..open_start);
        let index = self.push_operator_token(close_token, close_range);
        assert_eq!(index, close_index);
        index
    }

    fn on_open_token(
        &mut self,
        boundary: ExpressionBoundary,
        error: Option<ExpressionBoundaryError>,
        open_range: ByteRange,
    ) {
        let open_index = self.ast().next_index();
        self.push_open_expression(open_index, boundary, None);
        if boundary.is_required() {
            self.push_open_token(boundary, error, open_range);
        }
    }

    /// Tells whether this expression is needed for precedence reasons.
    /// Returns the popped expression, or None if the expression does not
    /// need to be inserted into the tree.
    fn pop_open_expression(&mut self) -> Option<OpenExpression> {
        let open_expression = self.open_expressions.pop().unwrap();
        match open_expression.boundary {
            // We don't need precedence groups unless they help.
            ExpressionBoundary::PrecedenceGroup => {
                match open_expression.infix {
                    Some((infix, infix_index)) => {
                        let parent_index = self.open_expressions.len() - 1;
                        let parent = &mut self.open_expressions[parent_index];
                        match parent.infix {
                            // If this parent has an infix and takes us as a right child, we are definitely needed.
                            Some((parent_infix, _)) if parent_infix.takes_right_child(infix) => {
                                Some(open_expression)
                            }
                            // If the parent has no infix, or if our infix is the new parent, we are not needed,
                            // but we do need to give our infix to the parent.
                            Some(_) | None => {
                                parent.infix = Some((infix, infix_index));
                                None
                            }
                        }
                    }
                    // We have no infix at all, so we aren't needed to resolve precedence. Yay!
                    None => None,
                }
            }
            ExpressionBoundary::CompoundTerm => {
                // We elide compound terms that have only prefixes and terms.
                let mut index = open_expression.open_index;
                while let PrefixOperator(_) = self.ast().expression_token(index) {
                    index += 1;
                }
                match self.ast().expression_token(index) {
                    Term(_) if index == self.ast().tokens.last_index() => None,
                    Open(_, _, delta) if index + delta == self.ast().tokens.last_index() => None,
                    Term(_) | Open(..) | PrefixOperator(_) => Some(open_expression),
                }
            }
            _ => {
                assert!(open_expression.boundary.is_required());
                Some(open_expression)
            }
        }
    }
}
