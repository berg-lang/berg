use syntax::AstData;
use syntax::{AstIndex, ExpressionBoundary, ExpressionBoundaryError, Fixity};
use syntax::ExpressionBoundary::*;
use parser::{ByteIndex, ByteRange, SourceRef};
use parser::binder::Binder;
use syntax::{InfixToken, Token};
use syntax::Token::*;

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
    infix: Option<(InfixToken, AstIndex)>,
    open_index: AstIndex,
    boundary: ExpressionBoundary,
}

impl<'a> Grouper<'a> {
    pub fn new(source: SourceRef<'a>) -> Self {
        Grouper {
            binder: Binder::new(source),
            open_expressions: Default::default(),
        }
    }

    pub fn ast(&self) -> &AstData<'a> {
        &self.binder.ast
    }
    pub fn ast_mut(&mut self) -> &mut AstData<'a> {
        &mut self.binder.ast
    }

    pub fn on_token(&mut self, token: Token, range: ByteRange) {
        match token {
            // Push the newly opened group onto open_expressions
            Open {
                boundary, error, ..
            } => self.on_open(boundary, error, range),
            // Delay the close token so that we can see the next infix.
            Close {
                boundary, error, ..
            } => self.on_close(boundary, error, range),

            // Infix tokens may have left->right or right->left precedence.
            InfixOperator(_) | InfixAssignment(_) | NewlineSequence | MissingInfix => {
                // Open or close PrecedenceGroups as necessary based on this infix.
                let infix = token.to_infix().unwrap();
                self.handle_precedence(infix, range.start);

                // Add the infix.
                let infix_index = self.push_token(token, range);
                // Set this as the last infix for future precedence checking
                self.open_expressions.last_mut().unwrap().infix = Some((infix, infix_index));
            }

            _ => {
                assert!(token.fixity() != Fixity::Infix);
                self.push_token(token, range);
            }
        }
    }

    pub fn on_source_end(self) -> AstData<'a> {
        self.binder.on_source_end()
    }

    fn handle_precedence(&mut self, next_infix: InfixToken, next_infix_start: ByteIndex) {
        if let Some((infix, index)) = self.open_expression().infix {
            // The normal order of things is that infixes run left to right.
            // If the next infix binds *tighter* than current, wrap it in a
            // "invisible parentheses" (a precedence subexpression).
            // e.g. 1+2*3 -> 1+2 -> 1+(2* ...
            // e.g. 1*2>3+4 -> 1*2>(3+ ...
            // e.g. 1+2>3*4 -> 1+2>(3* ...
            // e.g. 1>2+3*4 -> 1>(2+(3* ...
            if infix.takes_right_child(next_infix) {
                let boundary = PrecedenceGroup;
                let open_index = index + 1;
                self.open_expressions.push(OpenExpression {
                    open_index,
                    boundary,
                    infix: None,
                });
            } else {
                // If the current expression is precedence, and its *parent* doesn't
                // want the next infix as a child, we have to close off the invisible
                // parentheses. Repeat as necessary.
                // 1+2*3>4 = 1+(2*3 -> 1+(2*3)>...
                while self.open_expression().boundary == PrecedenceGroup {
                    if let Some((parent_infix, _)) = self.parent_expression().infix {
                        if parent_infix.takes_right_child(next_infix) {
                            break;
                        } else {
                            self.close(
                                next_infix_start..next_infix_start,
                                ExpressionBoundaryError::None,
                            );
                        }
                    }
                }
            }
        }
    }

    fn open_expression(&self) -> &OpenExpression {
        self.open_expressions.last().unwrap()
    }

    fn parent_expression(&self) -> &OpenExpression {
        &self.open_expressions[self.open_expressions.len() - 2]
    }

    fn on_close(
        &mut self,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        range: ByteRange,
    ) {
        // Close lesser subexpressions: File > Parentheses > CompoundTerm > PrecedenceGroup
        loop {
            use std::cmp::Ordering::*;
            let open_boundary = self.open_expression().boundary;
            match boundary.partial_cmp(&open_boundary).unwrap() {
                Greater => {
                    // Close and continue. Report "open without close" if parentheses get closed early
                    match open_boundary {
                        Source | Parentheses | CurlyBraces => self.close(
                            range.start..range.start,
                            ExpressionBoundaryError::OpenWithoutClose,
                        ),
                        CompoundTerm | PrecedenceGroup => {
                            self.close(range.start..range.start, ExpressionBoundaryError::None)
                        }
                        Root => unreachable!(),
                    }
                }
                Equal => {
                    self.close(range, error);
                    break;
                }
                Less => {
                    match boundary {
                        Source | Parentheses | CurlyBraces => {
                            let error = ExpressionBoundaryError::CloseWithoutOpen;
                            // Insert a fake open token to match the close token
                            let open_index = {
                                let parent = self.open_expression();
                                match parent.boundary {
                                    CompoundTerm | PrecedenceGroup => parent.open_index,
                                    Source | Parentheses | CurlyBraces => parent.open_index + 1,
                                    Root => unreachable!(),
                                }
                            };
                            self.insert_token_pair(open_index, boundary, error, range);
                        }
                        CompoundTerm | PrecedenceGroup => {}
                        Root => unreachable!(),
                    }
                    break;
                }
            }
        }
    }

    fn close(&mut self, range: ByteRange, error: ExpressionBoundaryError) {
        let expression = self.open_expressions.pop().unwrap();
        if expression.boundary == CompoundTerm
            && !(expression.infix.is_some() && self.open_expression().infix.is_some())
        {
            // Unnecessary CompoundTerms, we silently remove.
            // TODO report error on unnecessary parentheses like (a+b)+c? Would let user know the
            // grouping is fine as-is, and we have few enough precedences that parens aren't needed for clarity generally.
            return;
        }

        match expression.boundary {
            CompoundTerm | PrecedenceGroup => {
                self.insert_token_pair(expression.open_index, expression.boundary, error, range);
            }
            Source | Parentheses | CurlyBraces => {
                // If we're fixing a missing open/close { or (, we may be creating an *empty* one, which
                // require a MissingExpression between them.
                // TODO the comments are probably right we need to deal with missing open as well here.
                // Think it through and implement (or change the comment).
                if error == ExpressionBoundaryError::OpenWithoutClose {
                    self.push_token(MissingExpression, range.start..range.start);
                }

                self.push_close_token(&expression, error, range);
            }
            Root => unreachable!(),
        }
    }

    fn push_open_expression(
        &mut self,
        open_index: AstIndex,
        boundary: ExpressionBoundary,
        infix: Option<(InfixToken, AstIndex)>,
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

    fn on_open(
        &mut self,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        open_range: ByteRange,
    ) {
        let open_index = self.ast().next_index();
        self.push_open_expression(open_index, boundary, None);
        if match boundary {
            Source | CurlyBraces | Parentheses => true,
            _ => false,
        } {
            self.push_open_token(boundary, error, open_range);
        }
    }
}
