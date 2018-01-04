use ast::IdentifierIndex;
use source::parse_result::{ByteIndex, ByteRange, ParseResult};
use ast::{AstIndex, Variable, VariableIndex};
use ast::token::{ExpressionBoundary, ExpressionBoundaryError, Fixity, InfixToken, Token};
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use ast::identifiers::*;
use ast;

// Handles nesting and precedence: balances (), {}, and compound terms, and
// inserts "precedence groups," and removes compound terms and precedence
// groups where it can.
#[derive(Debug)]
pub(super) struct Grouper {
    open_expressions: Vec<OpenExpression>,
    scope: Vec<VariableIndex>,
    first_local_variable: usize,
}

#[derive(Debug)]
struct OpenExpression {
    infix: Option<(InfixToken, AstIndex)>,
    first_variable: usize,
    open_index: AstIndex,
    boundary: ExpressionBoundary,
}

impl Grouper {
    pub(super) fn new() -> Self {
        let scope = ast::root_variables()
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .collect();
        Grouper {
            open_expressions: Default::default(),
            scope,
            first_local_variable: 0,
        }
    }

    pub(super) fn variable_index(
        &mut self,
        name: IdentifierIndex,
        parse_result: &mut ParseResult,
    ) -> VariableIndex {
        // TODO fix parent references (:a = a + 1) so they work appropriately.
        let first_variable = match parse_result.tokens.last() {
            Some(&PrefixOperator(COLON)) => self.first_local_variable,
            _ => 0,
        };
        if let Some(variable) = self.scope[first_variable..]
            .iter()
            .rev()
            .find(|variable| parse_result.variables[**variable].name == name)
        {
            return *variable;
        }
        // We couldn't find it (or we exposed a new field). Declare it in local scope.
        let index = parse_result.variables.push(Variable { name });
        self.scope.push(index);
        index
    }

    pub(super) fn on_token(
        &mut self,
        token: Token,
        range: ByteRange,
        parse_result: &mut ParseResult,
    ) {
        match token {
            // Push the newly opened group onto open_expressions
            Open {
                boundary, error, ..
            } => self.on_open(boundary, error, range, parse_result),
            // Delay the close token so that we can see the next infix.
            Close {
                boundary, error, ..
            } => self.on_close(boundary, error, range, parse_result),

            // Infix tokens may have left->right or right->left precedence.
            InfixOperator(_) | InfixAssignment(_) | NewlineSequence | MissingInfix => {
                // Open or close PrecedenceGroups as necessary based on this infix.
                let infix = token.to_infix().unwrap();
                self.handle_precedence(infix, range.start, parse_result);

                // Add the infix.
                let infix_index = parse_result.push_token(token, range);
                // Set this as the last infix for future precedence checking
                self.open_expressions.last_mut().unwrap().infix = Some((infix, infix_index));
            }

            RawIdentifier(identifier) => {
                let token = VariableReference(self.variable_index(identifier, parse_result));
                self.on_token(token, range, parse_result)
            }

            _ => {
                assert!(token.fixity() != Fixity::Infix);
                parse_result.push_token(token, range);
            }
        }
    }

    fn handle_precedence(
        &mut self,
        next_infix: InfixToken,
        next_infix_start: ByteIndex,
        parse_result: &mut ParseResult,
    ) {
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
                    first_variable: self.scope.len(),
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
                                parse_result,
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
        parse_result: &mut ParseResult,
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
                            parse_result,
                            ExpressionBoundaryError::OpenWithoutClose,
                        ),
                        CompoundTerm | PrecedenceGroup => self.close(
                            range.start..range.start,
                            parse_result,
                            ExpressionBoundaryError::None,
                        ),
                    }
                }
                Equal => {
                    self.close(range, parse_result, error);
                    break;
                }
                Less => {
                    match boundary {
                        Source | Parentheses | CurlyBraces => {
                            // Insert a fake open token to match the close token
                            let open_index = {
                                let parent = self.open_expression();
                                match parent.boundary {
                                    CompoundTerm | PrecedenceGroup => parent.open_index,
                                    Source | Parentheses | CurlyBraces => parent.open_index + 1,
                                }
                            };
                            let start = parse_result.token_ranges[open_index].start;
                            let open_token = boundary
                                .placeholder_open_token(ExpressionBoundaryError::CloseWithoutOpen);
                            println!(
                                "Inserting fake {:?}: open_index = {}, start={}",
                                open_token, open_index, start
                            );
                            parse_result.insert_token(open_index, open_token, start..start);
                            self.open_expressions.push(OpenExpression {
                                open_index,
                                boundary,
                                infix: None,
                                first_variable: self.scope.len(),
                            });

                            self.close(
                                range,
                                parse_result,
                                ExpressionBoundaryError::CloseWithoutOpen,
                            );
                        }
                        CompoundTerm | PrecedenceGroup => {}
                    }
                    break;
                }
            }
        }
    }

    fn close(
        &mut self,
        range: ByteRange,
        parse_result: &mut ParseResult,
        error: ExpressionBoundaryError,
    ) {
        let expression = self.open_expressions.pop().unwrap();
        println!(
            "close({:?}) at {}: open index = {}",
            expression.boundary,
            parse_result.next_index(),
            expression.open_index
        );
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
                let start = parse_result.token_ranges[expression.open_index].start;
                let close_index = parse_result.next_index() + 1; // Have to add 1 due to the impending insert.
                let delta = close_index - expression.open_index;
                let open_token = Open {
                    boundary: expression.boundary,
                    delta,
                    error,
                };
                let close_token = Close {
                    boundary: expression.boundary,
                    delta,
                    error,
                };
                parse_result.insert_token(expression.open_index, open_token, start..start);
                parse_result.push_token(close_token, range);
            }
            Source | Parentheses | CurlyBraces => {
                // If we're fixing a missing open/close { or (, we may be creating an *empty* one, which
                // require a MissingExpression between them.
                // TODO the comments are probably right we need to deal with missing open as well here.
                // Think it through and implement (or change the comment).
                if error == ExpressionBoundaryError::OpenWithoutClose {
                    parse_result.push_token(MissingExpression, range.start..range.start);
                }

                let mut delta = parse_result.next_index() - expression.open_index;
                if let Open {
                    boundary,
                    delta: ref mut open_delta,
                    error: ref mut open_error,
                } = parse_result.tokens[expression.open_index]
                {
                    assert_eq!(boundary, expression.boundary);
                    *open_delta = delta;
                    *open_error = error;
                } else {
                    unreachable!()
                }
                let close_token = Close {
                    boundary: expression.boundary,
                    delta,
                    error,
                };
                parse_result.push_token(close_token, range);
            }
        }

        // If this is Source or CurlyBraces, we are closing a block and need to clear out any variables in scope.
        match expression.boundary {
            Source | CurlyBraces => self.scope.truncate(expression.first_variable),
            CompoundTerm | PrecedenceGroup | Parentheses => {}
        }
    }

    fn on_open(
        &mut self,
        boundary: ExpressionBoundary,
        error: ExpressionBoundaryError,
        open_range: ByteRange,
        parse_result: &mut ParseResult,
    ) {
        let open_index = parse_result.next_index();
        println!("OPEN {:?}: {}", boundary, open_index);
        self.open_expressions.push(OpenExpression {
            open_index,
            boundary,
            infix: None,
            first_variable: self.scope.len(),
        });
        match boundary {
            Source | Parentheses | CurlyBraces => {
                parse_result.push_token(boundary.placeholder_open_token(error), open_range);
            }
            // CompoundTerm and PrecedenceGroup typically don't end up in the AST, so we don't insert
            // them until we discover we have to.
            CompoundTerm | PrecedenceGroup => {}
        };
    }
}
