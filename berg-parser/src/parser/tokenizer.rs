use crate::syntax::{
    ast::Ast,
    bytes::{ByteIndex, ByteRange},
    identifiers::{APPLY, FOLLOWED_BY, NEWLINE_SEQUENCE},
    token::{ExpressionBoundary, ExpressionToken, OperatorToken, TermToken},
};
use OperatorToken::*;
use TermToken::*;
use WhitespaceState::*;

use super::{grouper::Grouper, sequencer::IndentLevel};

///
/// This builds up a valid expression from the incoming sequences.
///
/// It does these things:
///
/// 1. Inserting apply, newline sequence, and missing expression as appropriate
///    when two operators or two terms are next to each other.
/// 2. Opening and closing terms (series of tokens with no space between operators/operands).
/// 3. Handling indented blocks.
///
#[derive(Debug)]
pub struct Tokenizer {
    ///
    /// The grouper (where we send tokens when we produce them).
    ///
    pub grouper: Grouper,

    ///
    /// Whether the previous token was an operator that needs an operand, or not.
    ///
    pub prev_was_operator: bool,

    ///
    /// Where we are with respect to whitespace groupings (vertical text blocks with indent and
    /// horizontal expressions without space)
    ///
    whitespace_state: WhitespaceState,

    ///
    /// The indent level of all open indented text blocks.
    ///
    /// This has 4 open blocks at 0, 4, 8 and 12:
    ///
    /// ```text
    /// MyClass:
    ///     MyFunction:
    ///         if x == 10
    ///             if y == 20
    /// ```
    ///
    indented_blocks: Vec<(IndentLevel, ExpressionBoundary)>,
}

///
/// State respect to whitespace groupings.
///
/// This tells us whether we are on a new line, and whether we are inside a compact term (an expression
/// with no space).
///
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum WhitespaceState {
    ///
    /// An expression has already started on the current line, and we are in a compact term.
    ///
    /// ```text
    /// x = 10
    /// y = 1+...
    /// ```
    ///
    /// ```text
    /// x = 10
    /// y = 1+2...
    /// ```
    ///
    InTerm,
    ///
    /// An expression has already started on the current line, and we are not in a term.
    ///
    /// ```text
    /// print ...
    /// ```
    ///
    /// ```text
    /// x * ...
    /// ```
    ///
    /// ```text
    /// x*(...
    /// ```
    ///
    NotInTerm,
    ///
    /// We are at the start of a line with the same indent level as the last one.
    ///
    /// ```text
    /// a = 10
    /// ...
    /// ```
    ///
    /// ```text
    /// a = 10 +
    /// ...
    /// ```
    ///
    NextLine,
    ///
    /// We are at the start of a line *more* indented than the last one.
    ///
    /// ```text
    /// if x > 10
    ///    ...
    /// ```
    ///
    /// ```text
    /// if x > 10 &&
    ///   ...
    /// ```
    ///
    IndentedLine(IndentLevel),
}

impl Default for Tokenizer {
    fn default() -> Self {
        Tokenizer {
            grouper: Grouper::default(),
            prev_was_operator: true,
            whitespace_state: NotInTerm,
            indented_blocks: vec![(0.into(), ExpressionBoundary::Source)],
        }
    }
}

impl Tokenizer {
    pub fn ast(&self) -> &Ast {
        self.grouper.ast()
    }

    pub fn ast_mut(&mut self) -> &mut Ast {
        self.grouper.ast_mut()
    }

    pub fn in_term(&self) -> bool {
        self.whitespace_state == InTerm
    }

    // The start of source emits the "open source" token.
    pub fn on_source_start(&mut self, start: ByteIndex) {
        let open_token = ExpressionBoundary::Source.placeholder_open_token(None);
        self.emit_expression_token(open_token, start..start)
    }

    // The end of the source closes any open terms, just like space. Also emits "close source."
    pub fn on_source_end(mut self, end: ByteIndex) -> Ast {
        let close_token = ExpressionBoundary::Source.placeholder_close_token();
        self.close_term(end);
        self.emit_operator_token(close_token, end..end);
        self.grouper.on_source_end()
    }

    pub fn on_expression_token(&mut self, token: impl Into<ExpressionToken>, range: ByteRange) {
        assert!(range.start < range.end);
        let token = token.into();

        // If we need to insert an operator, insert either NEWLINE_SEQUENCE or FOLLOWED_BY.
        let prev_was_operator = self.prev_was_operator;
        if !self.prev_was_operator {
            let operator = match self.whitespace_state {
                // x\ny resolves to x NEWLINE_SEQUENCE y
                NextLine => NEWLINE_SEQUENCE,
                // x y resolves to x FOLLOWED BY y
                // (x)y resolves to (x) FOLLOWED_BY y
                // x(y) is handled in on_open() and resolves to x APPLY y
                // x\n<indent>y resolves to x FOLLOWED_BY <IndentedBlock> y </IndentedBlock>
                InTerm | NotInTerm | IndentedLine(_) => FOLLOWED_BY,
            };
            self.emit_operator_token(InfixOperator(operator), range.start..range.start);
        }

        // If this is a new indented line, open up a new indented block.
        // An important note: because this only happens in on_expression_token(), an indented line
        // that starts with an operator or comment will *not* open a new indented block. This is
        // intentional.
        if let IndentedLine(indent) = self.whitespace_state {
            let boundary = if prev_was_operator {
                ExpressionBoundary::IndentedExpression
            } else {
                ExpressionBoundary::IndentedBlock
            };
            let open_indent = boundary.placeholder_open_token(None);
            self.emit_expression_token(open_indent, range.start..range.start);
            self.indented_blocks.push((indent, boundary));
        }

        // If we weren't in a term, we are now!
        self.open_term(range.start);

        // Send the token
        self.emit_expression_token(token, range);
    }

    pub fn on_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        assert!(range.start < range.end);
        self.emit_operator_token(token, range);
    }

    ///
    /// Handle space, which closes compound terms.
    ///
    pub fn on_space(&mut self, range: ByteRange) {
        self.close_term(range.start);
    }

    ///
    /// Handle space, which closes any compound terms.
    ///
    pub fn on_comment(&mut self, range: ByteRange) {
        // Comment after a term closes it.
        // We don't have delimited comments, so a+/*comment*/b is impossible.
        self.close_term(range.start);
    }

    ///
    /// Handle new lines.
    ///
    /// This closes any open blocks with lower indent, and then prepares us to possibly create a
    /// new block (if the next token is an expression rather than an operator or comment).
    ///
    pub fn on_line_start(
        &mut self,
        start: ByteIndex,
        indent: IndentLevel,
        matching_indent: IndentLevel,
    ) {
        // Close any open indented blocks (at least one block will be equal to 0, and that will never be closed).
        let mut top = self.indented_blocks.last().unwrap();
        while indent < top.0 {
            let close_token = top.1.placeholder_close_token();
            self.emit_operator_token(close_token, start..start);
            self.indented_blocks.pop();
            top = self.indented_blocks.last().unwrap();
        }
        // Mark indented blocks with errors on mismatched indent (spaces vs tabs, etc.).
        if matching_indent < top.0 {
            let mismatch_level = self
                .indented_blocks
                .iter()
                .position(|(block_indent, _)| matching_indent < *block_indent)
                .unwrap();
            self.grouper.on_indent_mismatch(mismatch_level);
        }
        // Remember this indent; we'll start a new block if we ever get an expression token.
        if indent == top.0 {
            self.whitespace_state = NextLine;
        } else {
            assert!(indent > top.0);
            self.whitespace_state = IndentedLine(indent);
        }
    }

    // ( or {.
    pub fn on_open(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        // `f(x,y)` is f APPLY x, y (two arguments), to distinguish it from `f (x,y)`,
        // which is f with a single argument, the tuple `(x,y)`.
        //
        // For comparison, this will also be used for the distinction between
        // `f[1]` and `f [1]` (one is an indexer, the other means "call f with
        // the first arg being a single-element array `[1]`").
        if !self.prev_was_operator && self.in_term() && boundary == ExpressionBoundary::Parentheses
        {
            self.emit_operator_token(InfixOperator(APPLY), range.start..range.start);
        }

        // Otherwise, just put the open token like normal.
        self.on_expression_token(boundary.placeholder_open_token(None), range);

        // Inside the (, a new expression term is started.
        self.whitespace_state = NotInTerm;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    pub fn on_close(&mut self, boundary: ExpressionBoundary, range: ByteRange) {
        self.on_operator_token(boundary.placeholder_close_token(), range);
        self.whitespace_state = InTerm;
    }

    // ; , or :. If it is in a term, the term is closed before the separator.
    // Afterwards, we are looking to start a new term, so it's still closed.
    pub fn on_separator(&mut self, token: OperatorToken, range: ByteRange) {
        self.close_term(range.start);
        self.on_operator_token(token, range);
    }

    fn open_term(&mut self, index: ByteIndex) {
        if !self.in_term() {
            let open_term = ExpressionBoundary::CompoundTerm.placeholder_open_token(None);
            self.emit_expression_token(open_term, index..index);
            self.whitespace_state = InTerm;
        }
    }

    fn close_term(&mut self, index: ByteIndex) {
        if self.in_term() {
            self.whitespace_state = NotInTerm;
            let close_token = ExpressionBoundary::CompoundTerm.placeholder_close_token();
            self.emit_operator_token(close_token, index..index)
        }
    }

    fn emit_expression_token(&mut self, token: ExpressionToken, range: ByteRange) {
        self.grouper.on_expression_token(token, range);
        self.prev_was_operator = token.has_right_operand();
    }
    fn emit_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        if self.prev_was_operator {
            self.emit_expression_token(MissingExpression.into(), range.start..range.start);
        }
        self.grouper.on_operator_token(token, range);
        self.prev_was_operator = token.has_right_operand();
    }
}
