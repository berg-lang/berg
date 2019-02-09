use crate::parser::binder::Binder;
use crate::syntax::identifiers::COLON;
use crate::syntax::ExpressionBoundary::*;
use crate::syntax::Token::*;
use crate::syntax::{
    Ast, AstIndex, ByteRange, ExpressionBoundary, ExpressionBoundaryError, Fixity, Token,
};

// Handles nesting and precedence: balances (), {}, and compound terms, and
// inserts "precedence groups," and removes compound terms and precedence
// groups where it can.
#[derive(Debug)]
pub struct Grouper<'a> {
    binder: Binder<'a>,
    open_expressions: Vec<OpenExpression>,
}

#[derive(Debug)]
struct OpenExpression {
    infix: Option<(Token, AstIndex)>,
    open_index: AstIndex,
    boundary: ExpressionBoundary,
}

impl<'a> Grouper<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        Grouper {
            binder: Binder::new(ast),
            open_expressions: Default::default(),
        }
    }

    pub fn ast(&self) -> &Ast<'a> {
        &self.binder.ast
    }
    pub fn ast_mut(&mut self) -> &mut Ast<'a> {
        &mut self.binder.ast
    }

    pub fn on_token(&mut self, token: Token, range: ByteRange) {
        match token {
            // Push the newly opened group onto open_expressions
            Open {
                boundary, error, ..
            } => self.on_open_token(boundary, error, range),
            // Delay the close token so that we can see the next infix.
            Close {
                boundary, error, ..
            } => self.on_close_token(boundary, error, range),

            // Infix tokens may have left->right or right->left precedence.
            InfixOperator(_) | InfixAssignment(_) | NewlineSequence | Apply => {
                // Open or close PrecedenceGroups as necessary based on this infix.
                let range_end = range.end;
                // Close parent groups that don't want us as a child.
                while !self.open_expression_wants_child(token) {
                    self.close(range.start..range.start, ExpressionBoundaryError::None);
                }
                // Open a precedence group if it's needed.
                self.open_precedence_group_if_needed(token);

                // Add the infix token, lest we forget!
                let index = self.push_token(token, range);

                // Set this as the last infix of the current open expression.
                self.open_expressions.last_mut().unwrap().infix = Some((token, index));

                // If the operator is a colon, we open an auto block because :
                // is defined to create a block. It will be automatically closed
                // by the next close token or lower precedence operator.
                if let InfixOperator(COLON) = token {
                    self.on_open_token(
                        AutoBlock,
                        ExpressionBoundaryError::None,
                        range_end..range_end,
                    );
                }
            }

            _ => {
                assert!(token.fixity() != Fixity::Infix);
                self.push_token(token, range);
            }
        }
    }

    pub fn on_source_end(self) -> Ast<'a> {
        self.binder.on_source_end()
    }

    fn open_precedence_group_if_needed(&mut self, next_infix: Token) {
        // Close any parent precedence groups unless they want this infix as a child.
        // If we are a right child of the parent, we need to wrap ourselves
        // in an "invisible parentheses" (a precedence subexpression).
        // e.g. 1+2*3 -> 1+2 -> 1+(2* ...
        // e.g. 1*2>3+4 -> 1*2>(3+ ...
        // e.g. 1+2>3*4 -> 1+2>(3* ...
        // e.g. 1>2+3*4 -> 1>(2+(3* ...
        if let Some((infix, index)) = self.open_expression().infix {
            if infix.takes_right_child(next_infix) {
                self.open_expressions.push(OpenExpression {
                    open_index: index + 1,
                    boundary: PrecedenceGroup,
                    infix: None,
                });
            }
        }
    }

    fn open_expression_wants_child(&self, next_infix: Token) -> bool {
        use crate::syntax::ExpressionBoundary::*;
        let infix = match self.open_expression().boundary {
            // The autoblock wants whatever its *parent* infix wants.
            AutoBlock => self.open_expressions[self.open_expressions.len() - 2].infix,
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

    fn on_close_token(
        &mut self,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        range: ByteRange,
    ) {
        loop {
            use std::cmp::Ordering::*;
            let open_boundary = self.open_expression().boundary;
            match boundary.partial_cmp(&open_boundary).unwrap() {
                // If we are HIGHER priority than the current expression (e.g. "( ... }"), the top
                // expression must be closed even though it is unmatched.
                Greater => {
                    let error = if open_boundary.is_closed_automatically() {
                        ExpressionBoundaryError::None
                    } else {
                        ExpressionBoundaryError::OpenWithoutClose
                    };
                    self.close(range.start..range.start, error);
                }
                // If they are the same, then we treat them as the same exact pair. This closes
                // the boundary.
                Equal => {
                    self.close(range, error);
                    break;
                }
                // If we are LOWER priority than the current expression (e.g. "{ ... )"), the close
                // token is unmatched and will be opened right after the open of the parent expression
                // and ).
                Less => {
                    let error = if boundary.is_required() {
                        ExpressionBoundaryError::CloseWithoutOpen
                    } else {
                        ExpressionBoundaryError::None
                    };
                    // Get the next index after the parent (if the parent hasn't been inserted yet, we
                    // insert at the parent's intended location).
                    let open_index = if open_boundary.is_required() {
                        self.open_expression().open_index + 1
                    } else {
                        self.open_expression().open_index
                    };
                    self.insert_token_pair(open_index, boundary, error, range);
                    break;
                }
            }
        }
    }

    fn close(&mut self, range: ByteRange, error: ExpressionBoundaryError) {
        if let Some(expression) = self.pop() {
            if expression.boundary.is_required() {
                self.push_close_token(&expression, error, range);
            } else {
                self.insert_token_pair(expression.open_index, expression.boundary, error, range);
            }
        }
    }

    fn push_open_expression(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        infix: Option<(Token, AstIndex)>,
    ) {
        let open_expression = OpenExpression {
            open_index,
            boundary,
            infix,
        };
        self.open_expressions.push(open_expression);
    }

    fn insert_token(&mut self, index: AstIndex, token: Token, range: ByteRange) {
        self.binder.insert_token(index, token, range)
    }

    fn push_token(&mut self, token: Token, range: ByteRange) -> AstIndex {
        self.binder.push_token(token, range)
    }

    fn push_open_token(
        &mut self,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        open_range: ByteRange,
    ) -> AstIndex {
        let open_token = boundary.placeholder_open_token(error);
        self.push_token(open_token, open_range)
    }

    fn push_close_token(
        &mut self,
        expression: &OpenExpression,
        error: ExpressionBoundaryError,
        close_range: ByteRange,
    ) -> AstIndex {
        let close_index = self.ast().next_index();
        let delta = close_index - expression.open_index;

        // Update open index
        {
            let ast = self.ast_mut();
            match ast.tokens[expression.open_index] {
                Open {
                    boundary,
                    delta: ref mut open_delta,
                    error: ref mut open_error,
                } => {
                    assert_eq!(boundary, expression.boundary);
                    *open_delta = delta;
                    *open_error = error;
                }
                OpenBlock {
                    index,
                    delta: ref mut open_delta,
                    error: ref mut open_error,
                } => {
                    assert_eq!(ast.blocks[index].boundary, expression.boundary);
                    *open_delta = delta;
                    *open_error = error;
                }
                _ => unreachable!(),
            }
        }

        let close_token = Close {
            boundary: expression.boundary,
            delta,
            error,
        };
        let index = self.push_token(close_token, close_range);
        assert_eq!(close_index, index);
        index
    }

    fn insert_token_pair(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        close_range: ByteRange,
    ) -> AstIndex {
        let open_start = self.ast().token_ranges[open_index].start;
        let close_index = self.ast().next_index() + 1; // Have to add 1 due to the impending insert.
        let delta = close_index - open_index;
        let open_token = Open {
            boundary,
            delta,
            error,
        };
        let close_token = Close {
            boundary,
            delta,
            error,
        };
        self.insert_token(open_index, open_token, open_start..open_start);
        let index = self.push_token(close_token, close_range);
        assert_eq!(index, close_index);
        index
    }

    fn on_open_token(
        &mut self,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
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
    fn pop(&mut self) -> Option<OpenExpression> {
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
                while self.ast().tokens[index].fixity() == Fixity::Prefix {
                    index += 1;
                }
                match self.ast().tokens[index].fixity() {
                    Fixity::Term if index == self.ast().tokens.last_index() => None,
                    Fixity::Open
                        if index + self.ast().tokens[index].delta()
                            == self.ast().tokens.last_index() =>
                    {
                        None
                    }
                    _ => Some(open_expression),
                }
            }
            _ => {
                assert!(open_expression.boundary.is_required());
                Some(open_expression)
            }
        }
    }
}
