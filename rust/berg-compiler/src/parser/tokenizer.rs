use ast::token::{ExpressionBoundary, ExpressionBoundaryError, Token};
use ast::token::Token::*;
use source::parse_result::{ByteIndex, ByteRange, ParseResult};
use parser::grouper::Grouper;

// This builds up a valid expression from the incoming sequences, doing two things:
// 1. Inserting apply, newline sequence, and missing expression as appropriate
//    when two operators or two terms are next to each other.
// 2. Opening and closing terms (series of tokens with no space between operators/operands).
#[derive(Debug)]
pub(super) struct Tokenizer {
    pub(super) grouper: Grouper,
    pub(super) in_term: bool,
    pub(super) operator: bool,
    newline_start: ByteIndex,
    newline_length: u8,
}

impl Tokenizer {
    pub(super) fn new() -> Self {
        Tokenizer {
            grouper: Grouper::new(),
            in_term: false,
            operator: true,
            newline_start: ByteIndex(0),
            newline_length: 0,
        }
    }

    fn source_error(&self, parse_result: &ParseResult) -> ExpressionBoundaryError {
        if parse_result.open_error.is_some() {
            ExpressionBoundaryError::OpenError
        } else {
            ExpressionBoundaryError::None
        }
    }

    // The start of source emits the "open source" token.
    pub(super) fn on_source_start(&mut self, start: ByteIndex, parse_result: &mut ParseResult) {
        println!("on_source_start(in_term: {})", self.in_term);
        let mut open_token =
            ExpressionBoundary::Source.placeholder_open_token(self.source_error(parse_result));
        if parse_result.open_error.is_some() {
            if let Open { ref mut error, .. } = open_token {
                *error = ExpressionBoundaryError::OpenError;
            } else {
                unreachable!()
            }
        }
        self.emit_token(open_token, start..start, parse_result)
    }

    // The end of the source closes any open terms, just like space. Also emits "close source."
    pub(super) fn on_source_end(&mut self, end: ByteIndex, parse_result: &mut ParseResult) {
        println!("on_source_end(in_term: {})", self.in_term);
        let close_token =
            ExpressionBoundary::Source.placeholder_close_token(self.source_error(parse_result));
        self.close_term(end, parse_result);
        self.emit_token(close_token, end..end, parse_result)
    }

    // +, foo, 123. If a term hasn't started, this will start it.
    pub(super) fn on_term_token(
        &mut self,
        token: Token,
        range: ByteRange,
        parse_result: &mut ParseResult,
    ) {
        println!("on_term_token(in_term: {}): {:?}", self.in_term, token);
        assert!(range.start < range.end);
        self.open_term(range.start, parse_result);
        self.emit_token(token, range, parse_result);
    }

    // Space after a term closes it.
    pub(super) fn on_space(&mut self, start: ByteIndex, parse_result: &mut ParseResult) {
        println!("on_space(in_term: {})", self.in_term);
        self.close_term(start, parse_result)
    }

    // Newline is space, so it closes terms just like space. If the last line ended in a complete
    // expression, we may be about to create a newline sequence. Save the first newline until we know
    // whether the next real line is an operator (continuation) or a new expression.
    pub(super) fn on_newline(
        &mut self,
        start: ByteIndex,
        length: u8,
        parse_result: &mut ParseResult,
    ) {
        println!("on_newline(in_term: {})", self.in_term);
        self.close_term(start, parse_result);
        if !self.operator && self.newline_length == 0 {
            self.newline_start = start;
            self.newline_length = length;
        }
    }

    // ( or {. If the ( is after a space, opens a new term. But once we're in the ( a new term will
    // be started.
    pub(super) fn on_open(
        &mut self,
        boundary: ExpressionBoundary,
        range: ByteRange,
        parse_result: &mut ParseResult,
    ) {
        println!("on_open(in_term: {}): {:?}", self.in_term, boundary);
        assert!(range.start < range.end);
        let token = boundary.placeholder_open_token(ExpressionBoundaryError::None);
        self.open_term(range.start, parse_result);
        self.emit_token(token, range, parse_result);
        self.in_term = false;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    pub(super) fn on_close(
        &mut self,
        boundary: ExpressionBoundary,
        range: ByteRange,
        parse_result: &mut ParseResult,
    ) {
        assert!(range.start < range.end);
        let token = boundary.placeholder_close_token(ExpressionBoundaryError::None);
        self.close_term(range.start, parse_result);
        self.emit_token(token, range, parse_result);
        self.in_term = true;
    }

    // ; or :. If the : is in a term, it closes it. Afterwards, we are looking to start a new term,
    // so it's still closed.
    pub(super) fn on_separator(
        &mut self,
        token: Token,
        range: ByteRange,
        parse_result: &mut ParseResult,
    ) {
        println!("on_separator(in_term: {}): {:?}", self.in_term, token);
        assert!(range.start < range.end);
        self.close_term(range.start, parse_result);
        self.emit_token(token, range, parse_result);
    }

    fn open_term(&mut self, index: ByteIndex, parse_result: &mut ParseResult) {
        if !self.in_term {
            let open_token = ExpressionBoundary::CompoundTerm
                .placeholder_open_token(ExpressionBoundaryError::None);
            self.emit_token(open_token, index..index, parse_result);
            self.in_term = true;
        }
    }

    fn close_term(&mut self, index: ByteIndex, parse_result: &mut ParseResult) {
        if self.in_term {
            self.in_term = false;
            let close_token = ExpressionBoundary::CompoundTerm
                .placeholder_close_token(ExpressionBoundaryError::None);
            self.emit_token(close_token, index..index, parse_result)
        }
    }

    fn emit_token(&mut self, token: Token, range: ByteRange, parse_result: &mut ParseResult) {
        if self.operator {
            if token.has_left_operand() {
                self.emit_token(MissingExpression, range.start..range.start, parse_result);
            }
        } else if !token.has_left_operand() {
            if self.newline_length > 0 {
                let newline_start = self.newline_start;
                let newline_end = newline_start + (self.newline_length as usize);
                self.emit_token(NewlineSequence, newline_start..newline_end, parse_result);
            } else {
                self.emit_token(MissingInfix, range.start..range.start, parse_result);
            }
        }
        self.grouper.on_token(token, range, parse_result);
        self.operator = token.has_right_operand();
    }
}
