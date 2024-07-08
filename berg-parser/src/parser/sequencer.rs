use crate::syntax::{
        ast::Ast,
        bytes::{ByteIndex, ByteRange, ByteSlice},
        char_data::WhitespaceIndex,
        token::{ErrorTermError, ExpressionBoundary, InlineBlockLevel, RawErrorTermError},
    };
use berg_util::Delta;
use std::{borrow::Cow, cmp::min, str};
use CharType::*;
use ErrorTermError::*;
use ExpressionBoundary::*;

use super::{
    scanner::{CharType, Scanner},
    tokenizer::Tokenizer,
};

///
/// Chunks up the source into sequences: space, newlines, operators, etc.
/// Assigns each sequence a token type--term, prefix, postfix, infix or space.
/// Passes these to the Tokenizer to handle expression and whitespace rules.
///
/// In general, the sequencer chunks *character runs*--sequences containing all
/// of the same type of character. The most straightforward of these include:
///
/// | Integer | `1234` | A run of digit characters. | Term |
/// | Identifier | `ThisIsAnIdentifier` | A run of alphanumeric characters, or `_`. | Term |
/// | Operator | `+` `-` `*` `++` `+=` `<=>` `--->` | A run of operator characters. | Prefix if unbalanced like `+1`, postfix if unbalanced like If it's unbalanced like `+1` or `2*`, it's a postfix/prefix operator. Otherwise it's infix.
/// | Space | ` ` | A run of space characters. | Space |
/// | Whitespace | ` ` `\t` | A run of space and other whitespace characters. | Space |
/// | Unsupported | | A run of valid UTF-8 characters which aren't supported (such as emoji). | Term |
/// | Invalid | | A run of invalid UTF-8 bytes. | Term |
///
/// The sequencer also recognizes very specific, short character sequences, like
/// special operators and newlines, including:
///
/// | Separator | `;` `,` | Infix |
/// | Colon | `:` | Prefix if unbalanced like `:x`, otherwise infix.
/// | Open | `{` `(` | Open |
/// | Close | `}` `)` | Close |
/// | Newline | `\r` `\n` `\r\n` | Newlines are treated separately from other space, so that they can be counted for line #'s and possibly used to separate statements.
///
#[derive(Debug)]
pub struct Sequencer {
    /// The tokenizer to send sequences to.
    tokenizer: Tokenizer,
    /// Scans UTF-8 characters.
    scanner: Scanner,
    /// Current indent level.
    current_indent: IndentLevel,
    /// Whitespace for current indent level.
    current_indent_whitespace: Option<WhitespaceIndex>,
}

///
/// The amount of indent on a line.
///
pub type IndentLevel = Delta<ByteIndex>;

#[derive(Copy, Clone)]
pub struct Sequence<'s> {
    pub buffer: &'s ByteSlice,
    pub start: ByteIndex,
    pub end: ByteIndex,
}

#[derive(Copy, Clone)]
pub struct PartialSequence<'s> {
    pub buffer: &'s ByteSlice,
    pub start: ByteIndex,
    pub full_end: ByteIndex,
    pub partial_end: ByteIndex,
}

impl Sequencer {
    pub fn new(buffer: Cow<'static, ByteSlice>) -> Self {
        let tokenizer = Tokenizer::default();
        let scanner = Scanner::new(buffer);
        Sequencer {
            tokenizer,
            scanner,
            current_indent: 0.into(),
            current_indent_whitespace: None,
        }
    }

    pub fn parse(mut self) -> Ast {
        self.tokenizer.on_source_start(self.scanner.index);
        self.line_start();

        let mut start = self.scanner.index;
        loop {
            let char_type = self.scanner.next();
            match char_type {
                Digit => self.integer(start),
                Identifier => self.identifier(start),
                OtherOperator => self.operator(start, char_type),
                Equal => self.equal(start),
                Dash => self.dash(start),
                ComparisonOperatorStart => self.comparison_operator_start(start),
                Separator => self.separator(start),
                Colon => self.colon(start),
                OpenParen => self
                    .tokenizer
                    .on_open(Parentheses, self.scanner.utf8(start)),
                CloseParen => self
                    .tokenizer
                    .on_close(Parentheses, self.scanner.utf8(start)),
                OpenCurly => self
                    .tokenizer
                    .on_open(CurlyBraces, self.scanner.utf8(start)),
                CloseCurly => self
                    .tokenizer
                    .on_close(CurlyBraces, self.scanner.utf8(start)),
                Hash => self.comment(start),
                // Equal => self.equal(start),
                // Dash => self.dash(start),
                Newline => self.newline(start),
                LineEnding => self.line_ending(start),
                Space => self.space(start),
                HorizontalWhitespace => self.horizontal_whitespace(start),
                Unsupported => self.unsupported(start),
                InvalidUtf8 => self.invalid_utf8(start),
                Eof => break,
            };

            start = self.scanner.index;
        }

        assert!(start == self.scanner.index);
        assert!(self.scanner.at_end());

        self.tokenizer.on_source_end(self.scanner.index)
    }

    pub fn ast(&self) -> &Ast {
        self.tokenizer.ast()
    }

    fn utf8_syntax_error(&mut self, error: ErrorTermError, start: ByteIndex) {
        self.tokenizer
            .on_error_term(error, self.scanner.utf8(start))
    }

    fn raw_syntax_error(&mut self, error: RawErrorTermError, start: ByteIndex) {
        self.tokenizer.on_raw_error_term(
            error,
            self.scanner.bytes(start),
            start..self.scanner.index,
        )
    }

    fn integer(&mut self, start: ByteIndex) {
        self.scanner.next_while(Digit);
        if self.scanner.next_while(&CharType::is_identifier_middle) {
            return self.utf8_syntax_error(IdentifierStartsWithNumber, start);
        }
        self.tokenizer.on_integer(self.scanner.utf8(start));
    }

    fn identifier(&mut self, start: ByteIndex) {
        self.scanner.next_while(&CharType::is_identifier_middle);
        self.tokenizer.on_identifier(self.scanner.utf8(start));
    }

    fn term_is_about_to_end(&self) -> bool {
        let char_type = self.scanner.peek();
        char_type.is_whitespace()
            || char_type.is_close()
            || char_type.is_separator()
            || (char_type == Colon && !self.scanner.peek_at(1).is_always_right_operand())
    }

    fn operator(&mut self, start: ByteIndex, mut last_char_type: CharType) {
        while self.scanner.peek().is_operator() {
            last_char_type = self.scanner.next();
        }
        if last_char_type == Equal {
            self.emit_assignment_operator(start);
        } else {
            self.emit_operator(start);
        }
    }

    fn emit_assignment_operator(&mut self, start: ByteIndex) {
        let operator = self.scanner.partial_utf8(start, self.scanner.index - 1);
        self.tokenizer
            .on_assignment_operator(operator, self.term_is_about_to_end());
    }

    fn emit_operator(&mut self, start: ByteIndex) {
        self.tokenizer
            .on_operator(self.scanner.utf8(start), self.term_is_about_to_end());
    }

    fn emit_block_delimiter(&mut self, start: ByteIndex, level: InlineBlockLevel) {
        self.tokenizer
            .on_block_delimiter(level, self.scanner.range(start));
    }

    fn equal(&mut self, start: ByteIndex) {
        // = is an assignment operator
        if !self.scanner.peek().is_operator() {
            return self.emit_assignment_operator(start);
        }
        if self.scanner.next_if(Equal) {
            // === (3 or more equals) is a block delimiter
            let has_three_equals = self.scanner.next_while(Equal);
            if !self.scanner.peek().is_operator() {
                if has_three_equals {
                    // === and beyond is a block delimiter
                    return self.emit_block_delimiter(start, InlineBlockLevel::One);
                } else {
                    // == is a normal operator
                    return self.emit_operator(start);
                }
            }
        }
        self.operator(start, Equal)
    }

    fn dash(&mut self, start: ByteIndex) {
        if self.scanner.next_if(Dash)
            && self.scanner.next_while(Dash)
            && !self.scanner.peek().is_operator()
        {
            self.emit_block_delimiter(start, InlineBlockLevel::Two)
        } else {
            self.operator(start, Dash)
        }
    }

    fn comparison_operator_start(&mut self, start: ByteIndex) {
        if self.scanner.next_while(Equal) {
            if self.scanner.peek().is_operator() {
                // <==>, >==<, etc.
                self.operator(start, Equal)
            } else {
                // >=, >==, !=, !==, etc.
                self.emit_operator(start)
            }
        } else {
            // >>, <<, etc.
            self.operator(start, ComparisonOperatorStart)
        }
    }

    fn separator(&mut self, start: ByteIndex) {
        self.tokenizer.on_separator(self.scanner.utf8(start));
    }

    // Colon is, sadly, just a little ... special.
    // If we're succeeded by an operand, and we're not in a term ("1 + :a", "a :b"), we are a prefix.
    // If we're succeeded by an operand, and we're in a term, and we're preceded by an operator ("1+:a"), we are a prefix.
    // Else, we are separator. ("a:b", a:-b", "a: b", "a:")
    // See where the "operator" function calculates whether the term is about to end for the other
    // relevant silliness to ensure "a+:b" means "(a) + (:b)".
    fn colon(&mut self, start: ByteIndex) {
        self.tokenizer.on_colon(
            self.scanner.range(start),
            self.scanner.peek().is_always_right_operand(),
        )
    }

    fn newline(&mut self, start: ByteIndex) {
        self.tokenizer.on_space(start);
        self.line_start();
    }

    fn line_ending(&mut self, start: ByteIndex) {
        self.store_whitespace_in_char_data(start);
        self.tokenizer.on_space(start);
        self.line_start();
    }

    fn line_start(&mut self) {
        let start = self.scanner.index;
        self.tokenizer.ast_mut().char_data.line_starts.push(start);

        // Get the indent level.
        let indent_whitespace = self.read_space(start);

        // Send "indent" unless we're a blank line.
        if !self.scanner.peek().ends_line() {
            let indent = self.scanner.index - start;
            self.tokenizer.on_line_start(
                start,
                indent,
                self.matching_indent(indent, indent_whitespace),
            );
            self.current_indent = indent;
            self.current_indent_whitespace = indent_whitespace;
        }
    }

    // Get the matching indent level--the number of characters shared by indent and indent_whitespace.
    fn matching_indent(
        &self,
        indent: IndentLevel,
        indent_whitespace: Option<WhitespaceIndex>,
    ) -> IndentLevel {
        match (indent_whitespace, self.current_indent_whitespace) {
            // The old indent and new indent are entirely space characters.
            (None, None) => indent,
            // The old indent and new indent both have non-space characters.
            (Some(indent_whitespace), Some(current_whitespace)) => {
                let indent_whitespace = self.ast().whitespace_string(indent_whitespace).as_bytes();
                let current_whitespace =
                    self.ast().whitespace_string(current_whitespace).as_bytes();
                let current_whitespace =
                    &current_whitespace[0..min(indent_whitespace.len(), current_whitespace.len())];
                indent_whitespace
                    .iter()
                    .zip(current_whitespace.iter())
                    .position(|(a, b)| a != b)
                    .unwrap_or_else(|| indent.into())
                    .into()
            }
            // The old indent is all spaces, and the new indent has other space characters in it.
            // As long as the
            (Some(indent_whitespace), None) => {
                let indent_whitespace = self.ast().whitespace_string(indent_whitespace).as_bytes();
                let indent_whitespace =
                    &indent_whitespace[0..min(indent.into(), indent_whitespace.len())];
                indent_whitespace
                    .iter()
                    .position(|b| *b != b' ')
                    .unwrap_or_else(|| indent.into())
                    .into()
            }
            // The new indent is all spaces, and the old indent has other space characters in it.
            (None, Some(current_whitespace)) => {
                let current_whitespace =
                    self.ast().whitespace_string(current_whitespace).as_bytes();
                let current_whitespace =
                    &current_whitespace[0..min(indent.into(), current_whitespace.len())];
                current_whitespace
                    .iter()
                    .position(|b| *b != b' ')
                    .unwrap_or_else(|| indent.into())
                    .into()
            }
        }
    }

    fn read_space(&mut self, start: ByteIndex) -> Option<WhitespaceIndex> {
        if self.scanner.next_while(&CharType::is_horizontal_whitespace) {
            Some(self.store_whitespace_in_char_data(start))
        } else {
            None
        }
    }

    fn space(&mut self, start: ByteIndex) {
        self.read_space(start);
        self.tokenizer.on_space(start)
    }

    fn horizontal_whitespace(&mut self, start: ByteIndex) {
        self.scanner.next_while(&CharType::is_horizontal_whitespace);
        self.store_whitespace_in_char_data(start);
        self.tokenizer.on_space(start)
    }

    // # <comment>
    fn comment(&mut self, start: ByteIndex) {
        self.scanner.next_until(&CharType::ends_line);
        self.tokenizer
            .ast_mut()
            .char_data
            .append_comment(self.scanner.bytes(start), start);
        self.tokenizer.on_comment(start);
    }

    fn unsupported(&mut self, start: ByteIndex) {
        self.scanner.next_while(Unsupported);
        self.utf8_syntax_error(ErrorTermError::UnsupportedCharacters, start)
    }

    fn invalid_utf8(&mut self, start: ByteIndex) {
        self.scanner.next_while(InvalidUtf8);
        self.raw_syntax_error(RawErrorTermError::InvalidUtf8, start)
    }

    fn store_whitespace_in_char_data(&mut self, start: ByteIndex) -> WhitespaceIndex {
        let seq = self.scanner.utf8(start);
        self.tokenizer
            .ast_mut()
            .char_data
            .append_whitespace(seq.str(), seq.start)
    }
}

impl<'s> Sequence<'s> {
    pub fn str(&self) -> &'s str {
        unsafe { str::from_utf8_unchecked(&self.buffer[self.range()]) }
    }
    pub fn range(&self) -> ByteRange {
        self.start..self.end
    }
}

impl<'s> From<Sequence<'s>> for String {
    fn from(value: Sequence<'s>) -> Self {
        value.str().to_string()
    }
}

impl<'s> AsRef<str> for Sequence<'s> {
    fn as_ref(&self) -> &str {
        self.str()
    }
}

impl<'s> PartialSequence<'s> {
    pub fn into_full(self) -> Sequence<'s> {
        Sequence {
            buffer: self.buffer,
            start: self.start,
            end: self.full_end,
        }
    }
    pub fn partial_str(&self) -> &'s str {
        unsafe { str::from_utf8_unchecked(&self.buffer[self.start..self.partial_end]) }
    }
    pub fn range(&self) -> ByteRange {
        self.start..self.full_end
    }
}
