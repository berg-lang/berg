use crate::parser::Grouper;
use crate::syntax::ExpressionToken::*;
use crate::syntax::OperatorToken::*;
use crate::syntax::{Ast, ByteIndex, ByteRange, ExpressionBoundary, ExpressionBoundaryError, ExpressionToken, OperatorToken, Token};

// This builds up a valid expression from the incoming sequences, doing two things:
// 1. Inserting apply, newline sequence, and missing expression as appropriate
//    when two operators or two terms are next to each other.
// 2. Opening and closing terms (series of tokens with no space between operators/operands).
#[derive(Debug)]
pub struct Tokenizer<'a> {
    pub grouper: Grouper<'a>,
    pub in_term: bool,
    pub prev_was_operator: bool,
    newline_start: ByteIndex,
    newline_length: u8,
}

impl<'a> Tokenizer<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        Tokenizer {
            grouper: Grouper::new(ast),
            in_term: false,
            prev_was_operator: true,
            newline_start: ByteIndex(0),
            newline_length: 0,
        }
    }

    pub fn ast(&self) -> &Ast<'a> {
        self.grouper.ast()
    }
    pub fn ast_mut(&mut self) -> &mut Ast<'a> {
        self.grouper.ast_mut()
    }

    fn source_error(&self) -> ExpressionBoundaryError {
        if self.ast().source_open_error.is_some() {
            ExpressionBoundaryError::OpenError
        } else {
            ExpressionBoundaryError::None
        }
    }

    // The start of source emits the "open source" token.
    pub fn on_source_start(&mut self, start: ByteIndex) {
        let mut open_token = ExpressionBoundary::Source.placeholder_open_token(self.source_error());
        if self.ast().source_open_error.is_some() {
            if let OpenBlock { ref mut error, .. } = open_token {
                *error = ExpressionBoundaryError::OpenError;
            } else {
                unreachable!()
            }
        }
        self.emit_expression_token(open_token, start..start)
    }

    // The end of the source closes any open terms, just like space. Also emits "close source."
    pub fn on_source_end(mut self, end: ByteIndex) -> Ast<'a> {
        let close_token = ExpressionBoundary::Source.placeholder_close_token(self.source_error());
        self.close_term(end);
        self.emit_operator_token(close_token, end..end);
        self.grouper.on_source_end()
    }

    // Signifies this token is inside a compound term with no spaces, like "1+2".
    // If a term hasn't started, this will start it.
    pub fn on_term_token(&mut self, token: impl Into<Token>, range: ByteRange) {
        assert!(range.start < range.end);
        self.open_term(range.start);
        self.emit_token(token.into(), range);
    }

    // Space after a term closes it.
    pub fn on_space(&mut self, start: ByteIndex) {
        self.close_term(start);
    }

    // Newline is space, so it closes terms just like space. If the last line ended in a evaluate
    // expression, we may be about to create a newline sequence. Save the first newline until we know
    // whether the next real line is an operator (continuation) or a new expression.
    pub fn on_newline(&mut self, start: ByteIndex, length: u8) {
        self.close_term(start);
        if !self.prev_was_operator && self.newline_length == 0 {
            self.newline_start = start;
            self.newline_length = length;
        }
    }

    // ( or {. If the ( is after a space, opens a new term. But once we're in the ( a new term will
    // be started.
    pub fn on_open(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        assert!(range.start < range.end);
        let token = boundary.placeholder_open_token(ExpressionBoundaryError::None);
        self.open_term(range.start);
        self.emit_expression_token(token, range);
        self.in_term = false;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    pub fn on_close(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        assert!(range.start < range.end);
        let token = boundary.placeholder_close_token(ExpressionBoundaryError::None);
        self.close_term(range.start);
        self.emit_operator_token(token, range);
        self.in_term = true;
    }

    // ; , or :. If it is in a term, the term is closed before the separator.
    // Afterwards, we are looking to start a new term, so it's still closed.
    pub fn on_separator(&mut self, token: OperatorToken, range: ByteRange) {
        assert!(range.start < range.end);
        self.close_term(range.start);
        self.emit_operator_token(token, range);
    }

    fn open_term(&mut self, index: ByteIndex) {
        if !self.in_term {
            let open_token = ExpressionBoundary::CompoundTerm
                .placeholder_open_token(ExpressionBoundaryError::None);
            self.emit_expression_token(open_token, index..index);
            self.in_term = true;
        }
    }

    fn close_term(&mut self, index: ByteIndex) {
        if self.in_term {
            self.in_term = false;
            let close_token = ExpressionBoundary::CompoundTerm
                .placeholder_close_token(ExpressionBoundaryError::None);
            self.emit_operator_token(close_token, index..index)
        }
    }

    fn emit_token(&mut self, token: Token, range: ByteRange) {
        match token {
            Token::Expression(token) => self.emit_expression_token(token, range),
            Token::Operator(token) => self.emit_operator_token(token, range),
        }
    }
    fn emit_expression_token(&mut self, token: ExpressionToken, range: ByteRange) {
        if !self.prev_was_operator {
            if self.newline_length > 0 {
                let newline_start = self.newline_start;
                let newline_end = newline_start + (self.newline_length as usize);
                self.emit_operator_token(NewlineSequence, newline_start..newline_end);
            } else {
                self.emit_operator_token(Apply, range.start..range.start);
            }
        }
        self.grouper.on_expression_token(token, range);
        self.prev_was_operator = token.has_right_operand();
    }
    fn emit_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        if self.prev_was_operator {
            self.emit_expression_token(MissingExpression, range.start..range.start);
        }
        self.grouper.on_operator_token(token, range);
        self.prev_was_operator = token.has_right_operand();
    }
}
