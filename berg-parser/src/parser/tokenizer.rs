use crate::{
    identifiers::COLON,
    syntax::{
        ast::{LiteralIndex, RawLiteralIndex},
        bytes::{ByteIndex, ByteRange},
        identifiers::{self, IdentifierIndex, APPLY, FOLLOWED_BY, NEWLINE_SEQUENCE},
        token::{
            ErrorTermError, ExpressionBoundary, ExpressionToken, Fixity, InlineBlockLevel, OperatorToken, RawErrorTermError, TermToken
        },
    },
};
use berg_util::IndexedVec;
use string_interner::{backend::StringBackend, StringInterner};
use ExpressionToken::*;
use OperatorToken::*;
use TermToken::*;
use WhitespaceState::*;

use super::{
    grouper::Grouper,
    sequencer::{IndentLevel, PartialSequence, Sequence},
};

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
    pub whitespace_state: WhitespaceState,

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
    // ///
    // /// Delimited block state (whether we're in a first-level, second-level or third-level delimited block):
    // ///
    // /// ```text
    // /// MyClass
    // /// =======
    // ///
    // /// MyFunction
    // /// -------
    // ///
    // /// MyFunction2
    // /// -------
    // ///
    // /// MyClass2
    // /// =======
    // /// ...
    // /// ```
    // ///
    // delimited_block_state: (bool, bool)

    pub identifiers: StringInterner<StringBackend<IdentifierIndex>>,
    pub literals: StringInterner<StringBackend<LiteralIndex>>,
    pub raw_literals: IndexedVec<Vec<u8>, RawLiteralIndex>,
}

///
/// State respect to whitespace groupings.
///
/// This tells us whether we are on a new line, and whether we are inside a compact term (an expression
/// with no space).
///
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WhitespaceState {
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

impl Tokenizer {
    pub fn new() -> Self {
        let identifiers = identifiers::intern_all();
        Tokenizer {
            grouper: Grouper::new(),
            prev_was_operator: true,
            whitespace_state: NotInTerm,
            indented_blocks: vec![(0.into(), ExpressionBoundary::Source)],
            // delimited_block_state: (false, false)
            identifiers,
            literals: StringInterner::new(),
            raw_literals: Default::default(),
        }
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
    pub fn on_source_end(&mut self, end: ByteIndex) {
        let close_token = ExpressionBoundary::Source.placeholder_close_token();
        self.close_term(end);
        self.emit_operator_token(close_token, end..end);
        self.grouper.on_source_end();
    }

    pub fn on_error_term(&mut self, error: ErrorTermError, seq: Sequence) {
        let literal = self.intern_literal(seq);
        self.on_expression_token(ErrorTerm(error, literal), seq.range());
    }

    pub fn on_raw_error_term(&mut self, error: RawErrorTermError, bytes: &[u8], range: ByteRange) {
        let raw_literal = self.raw_literals.push(bytes.into());
        self.on_expression_token(RawErrorTerm(error, raw_literal), range);
    }

    pub fn on_integer(&mut self, seq: Sequence) {
        let literal = self.intern_literal(seq);
        self.on_expression_token(IntegerLiteral(literal), seq.range())
    }

    pub fn on_identifier(&mut self, seq: Sequence) {
        let identifier = self.intern_identifier(seq);
        self.on_expression_token(RawIdentifier(identifier), seq.range())
    }

    pub fn on_separator(&mut self, seq: Sequence) {
        let identifier = self.intern_identifier(seq);
        self.on_separator_token(InfixOperator(identifier), seq.range())
    }

    pub fn on_colon(&mut self, range: ByteRange, next_char_is_always_right_operand: bool) {
        if (!self.in_term() || self.prev_was_operator) && next_char_is_always_right_operand {
            self.on_expression_token(PrefixOperator(COLON), range);
        } else {
            self.on_separator_token(InfixOperator(COLON), range);
        }
    }

    fn on_expression_token(&mut self, token: impl Into<ExpressionToken>, range: ByteRange) {
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

    pub fn intern_identifier(
        &mut self,
        identifier: impl Into<String> + AsRef<str>,
    ) -> IdentifierIndex {
        self.identifiers.get_or_intern(identifier)
    }

    pub fn intern_literal(&mut self, literal: impl Into<String> + AsRef<str>) -> LiteralIndex {
        self.literals.get_or_intern(literal)
    }

    pub fn on_assignment_operator(&mut self, seq: PartialSequence, term_is_about_to_end: bool) {
        if self.operator_fixity(term_is_about_to_end) == Fixity::Infix {
            // If the infix operator is like a+b, it's inside the term. If it's
            // like a + b, it's outside (like a separator).
            let operator = self.intern_identifier(seq.partial_str());
            self.on_operator_token(InfixAssignment(operator), seq.range())
        } else {
            self.on_operator(seq.into_full(), term_is_about_to_end);
        }
    }

    pub fn on_block_delimiter(&mut self, level: InlineBlockLevel, range: ByteRange) {
        self.on_separator_token(InlineBlockDelimiter(level, range.end - range.start), range)
    }

    pub fn on_operator(&mut self, seq: Sequence, term_is_about_to_end: bool) {
        let operator = self.intern_identifier(seq.str());
        match self.operator_fixity(term_is_about_to_end) {
            Fixity::Postfix => self.on_operator_token(PostfixOperator(operator), seq.range()),
            Fixity::Prefix => self.on_expression_token(PrefixOperator(operator), seq.range()),
            Fixity::Infix => self.on_operator_token(InfixOperator(operator), seq.range()),
            Fixity::Close | Fixity::Open | Fixity::Term => unreachable!(),
        }
    }

    fn operator_fixity(&self, term_is_about_to_end: bool) -> Fixity {
        // If the term is about to end, this operator is postfix. i.e. "a? + 2"
        if self.in_term() && term_is_about_to_end {
            Fixity::Postfix

        // If we're *not* in a term, and there is something else right after the
        // operator, it is prefix. i.e. "+1"
        } else if !self.in_term() && !term_is_about_to_end {
            Fixity::Prefix

        // Otherwise, it's infix. i.e. "1+2" or "1 + 2"
        } else {
            Fixity::Infix
        }
    }

    // ; , or :. If it is in a term, the term is closed before the separator.
    // Afterwards, we are looking to start a new term, so it's still closed.
    fn on_separator_token(&mut self, token: OperatorToken, range: ByteRange) {
        self.close_term(range.start);
        self.on_operator_token(token, range);
    }

    fn on_operator_token(&mut self, token: OperatorToken, range: ByteRange) {
        assert!(range.start < range.end);
        self.emit_operator_token(token, range);
    }

    ///
    /// Handle space, which closes compound terms.
    ///
    pub fn on_space(&mut self, start: ByteIndex) {
        self.close_term(start);
    }

    ///
    /// Handle space, which closes any compound terms.
    ///
    pub fn on_comment(&mut self, start: ByteIndex) {
        // Comment after a term closes it.
        // We don't have delimited comments, so a+/*comment*/b is impossible.
        self.close_term(start);
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
    pub fn on_open(&mut self, boundary: ExpressionBoundary, seq: Sequence) {
        // `f(x,y)` is f APPLY x, y (two arguments), to distinguish it from `f (x,y)`,
        // which is f with a single argument, the tuple `(x,y)`.
        //
        // For comparison, this will also be used for the distinction between
        // `f[1]` and `f [1]` (one is an indexer, the other means "call f with
        // the first arg being a single-element array `[1]`").
        if !self.prev_was_operator && self.in_term() && boundary == ExpressionBoundary::Parentheses
        {
            self.emit_operator_token(InfixOperator(APPLY), seq.start..seq.start);
        }

        // Otherwise, just put the open token like normal.
        self.on_expression_token(boundary.placeholder_open_token(None), seq.range());

        // Inside the (, a new expression term is started.
        self.whitespace_state = NotInTerm;
    }

    // ) or }. If the ) is in a term, it closes it. After the ), we are definitely in a term,
    // however--part of the outer (...) term.
    pub fn on_close(&mut self, boundary: ExpressionBoundary, seq: Sequence) {
        self.on_operator_token(boundary.placeholder_close_token(), seq.range());
        self.whitespace_state = InTerm;
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
