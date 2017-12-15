use ast::{Tokens,TokenRanges};
use ast::token::{ExpressionBoundary,Token};
use ast::token::Token::*;
use compiler::Compiler;
use compiler::source_data::{ByteIndex,ByteRange,SourceIndex};
use parser::grouper::Grouper;

// This builds up a valid expression from the incoming sequences, doing two things:
// 1. Inserting apply, newline sequence, and missing expression as appropriate
//    when two operators or two terms are next to each other.
// 2. Opening and closing terms (series of tokens with no space between operators/operands).
#[derive(Debug)]
pub(super) struct Tokenizer<'p,'c:'p> {
    ast_builder: Grouper<'p,'c>,
    pub(super) in_term: bool,
    pub(super) operator: bool,
    newline_start: ByteIndex,
    newline_length: u8,
}

impl<'p,'c:'p> Tokenizer<'p,'c> {
    pub(super) fn new(ast_builder: Grouper<'p,'c>) -> Self {
        Tokenizer {
            ast_builder,
            in_term: false,
            operator: true,
            newline_start: ByteIndex(0),
            newline_length: 0,
        }
    }

    // The start of source emits the "open source" token.
    pub(super) fn on_source_start(&mut self, start: ByteIndex) {
        println!("on_source_start(in_term: {})", self.in_term);
        let token = ExpressionBoundary::Source.placeholder_open_token();
        self.emit_token(token, start..start)
    }

    // The end of the source closes any open terms, just like space. Also emits "close source."
    pub(super) fn on_source_end(&mut self, end: ByteIndex) {
        println!("on_source_end(in_term: {})", self.in_term);
        self.close_term(end);
        let close_token = ExpressionBoundary::Source.placeholder_close_token();
        self.emit_token(close_token, end..end)
    }

    // +, foo, 123. If a term hasn't started, this will start it.
    pub(super) fn on_term_token(&mut self, token: Token, range: ByteRange) {
        println!("on_term_token(in_term: {}): {:?}", self.in_term, token);
        assert!(range.start < range.end);
        self.open_term(range.start);
        self.emit_token(token, range);
    }

    // Space after a term closes it.
    pub(super) fn on_space(&mut self, start: ByteIndex) {
        println!("on_space(in_term: {})", self.in_term);
        self.close_term(start)
    }

    // Newline is space, so it closes terms just like space. If the last line ended in a complete
    // expression, we may be about to create a newline sequence. Save the first newline until we know
    // whether the next real line is an operator (continuation) or a new expression.
    pub(super) fn on_newline(&mut self, start: ByteIndex, length: u8) {
        println!("on_newline(in_term: {})", self.in_term);
        self.close_term(start);
        if !self.operator && self.newline_length == 0 {
            self.newline_start = start;
            self.newline_length = length;
        }
    }

    // ( or {. If the ( is after a space, opens a new term. But once we're in the ( a new term will
    // be started.
    pub(super) fn on_open(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        println!("on_open(in_term: {}): {:?}", self.in_term, boundary);
        assert!(range.start < range.end);
        let token = boundary.placeholder_open_token();
        self.open_term(range.start);
        self.emit_token(token, range);
        self.in_term = false;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    pub(super) fn on_close(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        println!("on_close(in_term: {}): {:?}", self.in_term, boundary);
        assert!(range.start < range.end);
        let token = boundary.placeholder_close_token();
        self.close_term(range.start);
        self.emit_token(token, range);
        self.in_term = true;
    }

    // ; or :. If the : is in a term, it closes it. Afterwards, we are looking to start a new term,
    // so it's still closed.
    pub(super) fn on_separator(&mut self, token: Token, range: ByteRange) {
        println!("on_separator(in_term: {}): {:?}", self.in_term, token);
        assert!(range.start < range.end);
        self.close_term(range.start);
        self.emit_token(token, range);
    }

    pub(super) fn compiler(&self) -> &Compiler<'c> {
        self.ast_builder.compiler
    }

    pub(super) fn source(&self) -> SourceIndex {
        self.ast_builder.source
    }


    pub(super) fn complete(self) -> (Tokens,TokenRanges) {
        self.ast_builder.complete()
    }

    fn open_term(&mut self, index: ByteIndex) {
        if !self.in_term {
            self.emit_token(ExpressionBoundary::CompoundTerm.placeholder_open_token(), index..index);
            self.in_term = true;
        }
    }

    fn close_term(&mut self, index: ByteIndex) {
        if self.in_term {
            self.in_term = false;
            self.emit_token(ExpressionBoundary::CompoundTerm.placeholder_close_token(), index..index)
        }
    }

    fn emit_token(&mut self, token: Token, range: ByteRange) {
        if self.operator {
            if token.has_left_operand() {
                self.emit_token(MissingExpression, range.start..range.start);
            }
        } else if !token.has_left_operand() {
            if self.newline_length > 0 {
                let newline_start = self.newline_start;
                let newline_end = newline_start + (self.newline_length as usize);
                self.emit_token(NewlineSequence, newline_start..newline_end);
            } else {
                self.emit_token(MissingInfix, range.start..range.start);
            }
        }
        self.ast_builder.on_token(token, range);
        self.operator = token.has_right_operand();
    }
}
