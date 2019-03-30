use crate::parser::sequencer::IndentLevel;
use crate::parser::Grouper;
use crate::syntax::OperatorToken::*;
use crate::syntax::TermToken::*;
use crate::syntax::{Ast, ByteIndex, ByteRange, ExpressionBoundary, ExpressionBoundaryError, ExpressionToken, OperatorToken};
use crate::syntax::identifiers::{NEWLINE_SEQUENCE, APPLY};

// This builds up a valid expression from the incoming sequences, doing two things:
// 1. Inserting apply, newline sequence, and missing expression as appropriate
//    when two operators or two terms are next to each other.
// 2. Opening and closing terms (series of tokens with no space between operators/operands).
#[derive(Debug)]
pub struct Tokenizer<'a> {
    pub grouper: Grouper<'a>,
    pub in_term: bool,
    pub prev_was_operator: bool,
    pub at_line_start: bool,
}

impl<'a> Tokenizer<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        Tokenizer {
            grouper: Grouper::new(ast),
            in_term: false,
            prev_was_operator: true,
            at_line_start: true,
        }
    }

    pub fn ast(&self) -> &Ast<'a> {
        self.grouper.ast()
    }
    pub fn ast_mut(&mut self) -> &mut Ast<'a> {
        self.grouper.ast_mut()
    }

    fn source_error(&self) -> Option<ExpressionBoundaryError> {
        if self.ast().source_open_error.is_some() {
            Some(ExpressionBoundaryError::OpenError)
        } else {
            None
        }
    }

    // The start of source emits the "open source" token.
    pub fn on_source_start(&mut self, start: ByteIndex) {
        let open_token = ExpressionBoundary::Source.placeholder_open_token(self.source_error());
        self.emit_expression_token(open_token, start..start)
    }

    // The end of the source closes any open terms, just like space. Also emits "close source."
    pub fn on_source_end(mut self, end: ByteIndex) -> Ast<'a> {
        let close_token = ExpressionBoundary::Source.placeholder_close_token();
        self.close_term(end);
        self.emit_operator_token(close_token, end..end);
        self.grouper.on_source_end()
    }

    // Signifies this token is inside a compound term with no spaces, like "1+2".
    // If a term hasn't started, this will start it.
    pub fn on_term_token(&mut self, token: impl Into<ExpressionToken>, range: ByteRange) {
        assert!(range.start < range.end);
        self.open_term(range.start);
        self.emit_expression_token(token.into(), range);
    }

    pub fn on_term_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        assert!(range.start < range.end);
        self.open_term(range.start);
        self.emit_operator_token(token, range);
    }

    ///
    /// Handle space.
    ///
    pub fn on_space(&mut self, range: ByteRange) {
        self.close_term(range.start);
    }

    pub fn on_comment(&mut self, range: ByteRange) {
        // Comment after a term closes it.
        // We don't have delimited comments, so a+/*comment*/b is impossible.
        self.close_term(range.start);
    }

    ///
    /// Handle newlines (tells us we want to do a newline sequence)
    /// 
    pub fn on_line_ending(&mut self, range: ByteRange) {
        self.close_term(range.start);
        self.at_line_start = true;
    }

    ///
    /// Handle indenting into a new block.
    /// 
    pub fn on_indent(&mut self, _indent: IndentLevel, _indented_from: IndentLevel) {
    }

    pub fn on_undent(&mut self, _indent: IndentLevel, _undented_from: IndentLevel) {
    }

    pub fn on_indent_mismatch(&mut self, _indent: IndentLevel, _mismatch_at: IndentLevel) {
    }

    // ( or {. If the ( is after a space, opens a new term. But once we're in the ( a new term will
    // be started.
    pub fn on_open(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        assert!(range.start < range.end);
        self.open_term(range.start);
        self.emit_expression_token(boundary.placeholder_open_token(None), range);
        self.in_term = false;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    pub fn on_close(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        assert!(range.start < range.end);
        let token = boundary.placeholder_close_token();
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
                .placeholder_open_token(None);
            self.emit_expression_token(open_token, index..index);
            self.in_term = true;
        }
    }

    fn close_term(&mut self, index: ByteIndex) {
        if self.in_term {
            self.in_term = false;
            let close_token = ExpressionBoundary::CompoundTerm
                .placeholder_close_token();
            self.emit_operator_token(close_token, index..index)
        }
    }

    fn emit_expression_token(&mut self, token: ExpressionToken, range: ByteRange) {
        if !self.prev_was_operator {
            if self.at_line_start {
                self.emit_operator_token(InfixOperator(NEWLINE_SEQUENCE), range.start..range.start);
            } else {
                self.emit_operator_token(InfixOperator(APPLY), range.start..range.start);
            }
        }
        self.grouper.on_expression_token(token, range);
        self.prev_was_operator = token.has_right_operand();
        self.at_line_start = false;
    }
    fn emit_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        if self.prev_was_operator {
            self.emit_expression_token(MissingExpression.into(), range.start..range.start);
        }
        self.grouper.on_operator_token(token, range);
        self.prev_was_operator = token.has_right_operand();
        self.at_line_start = false;
    }
}
