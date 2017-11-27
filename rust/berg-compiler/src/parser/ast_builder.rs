use compiler::Compiler;
use compile_errors;
use compiler::source_data::{ByteIndex,ByteRange,SourceIndex};
use ast::AstIndex;
use ast::token::{ExpressionBoundary,Fixity,InfixToken,Token};
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use indexed_vec::{Delta,IndexedVec};

#[derive(Debug)]
pub(super) struct AstBuilder<'p,'c:'p> {
    pub(super) compiler: &'p Compiler<'c>,
    pub(super) source: SourceIndex,
    tokens: IndexedVec<Token,AstIndex>,
    token_ranges: IndexedVec<ByteRange,AstIndex>,
    open_expressions: Vec<OpenExpression>,
}

#[derive(Debug)]
struct OpenExpression {
    open_index: AstIndex,
    boundary: ExpressionBoundary,
    infix: Option<(InfixToken,AstIndex)>,
}

impl<'p,'c:'p> AstBuilder<'p,'c> {
    pub(super) fn new(compiler: &'p Compiler<'c>, source: SourceIndex) -> Self {
        AstBuilder {
            compiler,
            source,
            tokens: Default::default(),
            token_ranges: Default::default(),
            open_expressions: Default::default(),
        }
    }

    pub(super) fn complete(self) -> (IndexedVec<Token,AstIndex>, IndexedVec<ByteRange,AstIndex>) {
        assert!(self.open_expressions.is_empty());
        (self.tokens, self.token_ranges)
    }

    pub(super) fn on_token(&mut self, token: Token, range: ByteRange) {
        println!("TOKEN {:?}", token);
        match token {
            // Push the newly opened group onto open_expressions
            Open(boundary, _) => self.on_open(boundary, range),
            // Delay the close token so that we can see the next infix.
            Close(boundary, _) => self.on_close(boundary, range),

            // Infix tokens may have left->right or right->left precedence.
            InfixOperator(_)|InfixAssignment(_)|NewlineSequence|MissingInfix => {
                // Open or close PrecedenceGroups as necessary based on this infix.
                let infix = token.to_infix().unwrap();
                self.handle_precedence(infix, range.start);

                // Add the infix.
                let infix_index = self.push(token, range);
                // Set this as the last infix for future precedence checking
                self.open_expressions.last_mut().unwrap().infix = Some((infix, infix_index));
            },

            _ => {
                assert!(token.fixity() != Fixity::Infix);
                self.push(token, range);
            },
        }
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
                let open_index = index+1;
                self.open_expressions.push(OpenExpression { open_index, boundary, infix: None });

            } else {
                // If the current expression is precedence, and its *parent* doesn't
                // want the next infix as a child, we have to close off the invisible
                // parentheses. Repeat as necessary.
                // 1+2*3>4 -> 1+(2*3) -> 1+(2*3)> ...
                while self.open_expression().boundary == PrecedenceGroup {
                    match self.parent_expression().infix {
                        Some((parent_infix,_)) if !parent_infix.takes_right_child(next_infix) => self.close(next_infix_start..next_infix_start),
                        _ => break,
                    }
                }
            }
        }
    }

    fn open_expression(&self) -> &OpenExpression {
        self.open_expressions.last().unwrap()
    }

    fn parent_expression(&self) -> &OpenExpression {
        &self.open_expressions[self.open_expressions.len()-2]
    }

    fn on_close(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        // Close lesser subexpressions: File > Parentheses > CompoundTerm > PrecedenceGroup
        loop {
            use std::cmp::Ordering::*;
            let open_boundary = self.open_expression().boundary;
            match boundary.partial_cmp(&open_boundary).unwrap() {
                Greater => {
                    // Close and continue. Report "open without close" if parentheses get closed early
                    match open_boundary {
                        Parentheses => {
                            let source = self.source;
                            let open_range = self.token_ranges[self.open_expression().open_index].clone();
                            let close = String::from(")");
                            self.compiler.report(compile_errors::OpenWithoutClose { source, open_range, close });
                        },
                        CompoundTerm|PrecedenceGroup|File => {},
                    }
                    self.close(range.start..range.start);
                },
                Equal => {
                    self.close(range);
                    break;
                },
                Less => {
                    match boundary {
                        Parentheses => {
                            let source = self.source;
                            let close_range = range.clone();
                            let open = String::from("(");
                            self.compiler.report(compile_errors::CloseWithoutOpen { source, close_range, open });
                        },
                        CompoundTerm|PrecedenceGroup|File => {},
                    }
                    break;
                }
            }
        }
    }

    fn close(&mut self, range: ByteRange) {
        match self.open_expression().boundary {
            File => self.actually_close(range),
            PrecedenceGroup => {
                // PrecedenceGroups are already known to be necessary by virtue of how we insert them.
                self.actually_close(range);
            },
            Parentheses|CompoundTerm => {
                // If we have an infix and there is a previous (parent) infix, the parentheses
                // are necessary so we can be its right child.
                // Otherwise, they are redundant (infix is processed left to right always).
                if self.open_expression().infix.is_some() && self.parent_expression().infix.is_some() {
                    self.actually_close(range);
                } else {
                    self.close_unnecessary(range);
                }
            },
        }
    }

    fn actually_close(&mut self, range: ByteRange) {
        let expression = self.open_expressions.pop().unwrap();
        match expression.boundary {
            File => {}, // Popping the expression is enough.
            CompoundTerm|PrecedenceGroup => {
                let start = self.token_ranges[expression.open_index].start;
                let close_index = self.next_index()+1; // Have to add 1 due to the impending insert.
                let delta = close_index-expression.open_index;
                self.insert(expression.open_index, Open(expression.boundary, delta), start..start);
                self.push(Close(expression.boundary, delta), range);
            },
            Parentheses => {
                let mut delta = self.next_index()-expression.open_index;
                // If we're fixing a missing open/close parentheses, we may be creating *empty* parens, which
                // require a MissingExpression between them.
                if delta == 1 { self.push(MissingExpression, range.start..range.start); delta = Delta(AstIndex(2)); }
                if let Open(boundary, ref mut open_delta) = self.tokens[expression.open_index] {
                    assert!(boundary == expression.boundary);
                    *open_delta = delta;
                } else {
                    unreachable!()
                }
                self.push(Close(expression.boundary, delta), range);
            },
        }
    }

    fn close_unnecessary(&mut self, range: ByteRange) {
        if self.open_expression().boundary == Parentheses {
            self.report_unnecessary_parentheses(&range);
            self.actually_close(range);
        } else {
            self.open_expressions.pop();
        }
    }

    fn report_unnecessary_parentheses(&mut self, _range: &ByteRange) {
        // TODO this is where you would warn!
    }

    fn on_open(&mut self, boundary: ExpressionBoundary, open_range: ByteRange) {
        let open_index = self.next_index();
        self.open_expressions.push(OpenExpression { open_index, boundary, infix: None });
        match boundary {
            Parentheses => { self.push(boundary.placeholder_open_token(), open_range); },
            // CompoundTerm and PrecedenceGroup typically don't end up in the AST, so we don't insert
            // them until we discover we have to.
            CompoundTerm|PrecedenceGroup|File => {}
        };
    }

    fn push(&mut self, token: Token, range: ByteRange) -> AstIndex {
        self.tokens.push(token);
        self.token_ranges.push(range)
    }

    fn insert(&mut self, index: AstIndex, token: Token, range: ByteRange) {
        self.tokens.insert(index, token);
        self.token_ranges.insert(index, range);
    }

    fn next_index(&self) -> AstIndex { self.tokens.len() }
}
