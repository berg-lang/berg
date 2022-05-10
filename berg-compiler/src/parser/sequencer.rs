use crate::parser::sequencer::ByteType::*;
use crate::parser::sequencer::CharType::*;
use crate::parser::Tokenizer;
use crate::syntax::ErrorTermError::*;
use crate::syntax::ExpressionBoundary::*;
use crate::syntax::ExpressionToken::*;
use crate::syntax::OperatorToken::*;
use crate::syntax::TermToken::*;
use crate::syntax::{
    Ast, ByteIndex, ByteRange, ByteSlice, ErrorTermError, IdentifierIndex, LiteralIndex,
    RawErrorTermError, WhitespaceIndex,
};
use crate::util::indexed_vec::Delta;
use std::cmp::min;
use std::str;

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
pub struct Sequencer<'a, 'p> {
    /// The tokenizer to send sequences to.
    tokenizer: Tokenizer<'a>,
    /// Scans UTF-8 characters.
    scanner: Scanner<'p>,
    /// Current indent level.
    current_indent: IndentLevel,
    /// Whitespace for current indent level.
    current_indent_whitespace: Option<WhitespaceIndex>,
}

///
/// Scans UTF-8 identifying characters.
///
#[derive(Debug, Clone)]
struct Scanner<'p> {
    /// The buffer we're scanning.
    buffer: &'p ByteSlice,
    /// The index of the next byte to read from the buffer.
    index: ByteIndex,
}

///
/// The amount of indent on a line.
///
pub type IndentLevel = Delta<ByteIndex>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CharType {
    Digit,
    Identifier,
    Operator,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    Separator,
    Colon,
    Hash,
    Newline,
    LineEnding,
    Space,
    HorizontalWhitespace,
    Unsupported,
    InvalidUtf8,
    Eof,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ByteType {
    Char(CharType),
    CarriageReturn,
    Utf8LeadingByte(Delta<ByteIndex>),
}

impl<'a, 'p> Sequencer<'a, 'p> {
    pub fn new(ast: Ast<'a>, buffer: &'p ByteSlice) -> Self {
        let tokenizer = Tokenizer::new(ast);
        let scanner = Scanner::new(buffer);
        Sequencer {
            tokenizer,
            scanner,
            current_indent: 0.into(),
            current_indent_whitespace: None,
        }
    }

    pub fn parse(mut self) -> Ast<'a> {
        self.tokenizer.on_source_start(self.scanner.index);
        self.line_start();

        let mut start = self.scanner.index;
        loop {
            let char_type = self.scanner.next();
            match char_type {
                Digit => self.integer(start),
                Identifier => self.identifier(start),
                Operator => self.operator(start),
                Separator => self.separator(start),
                Colon => self.colon(start),
                OpenParen => self.tokenizer.on_open(Parentheses, self.range(start)),
                CloseParen => self.tokenizer.on_close(Parentheses, self.range(start)),
                OpenCurly => self.tokenizer.on_open(CurlyBraces, self.range(start)),
                CloseCurly => self.tokenizer.on_close(CurlyBraces, self.range(start)),
                Hash => self.comment(start),
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
        assert!(self.scanner.index == self.scanner.buffer.len());

        self.ast_mut().char_data.size = self.scanner.index;

        self.tokenizer.on_source_end(self.scanner.index)
    }

    fn range(&self, start: ByteIndex) -> ByteRange {
        start..self.scanner.index
    }
    fn bytes(&self, start: ByteIndex) -> &'p [u8] {
        &self.buffer()[self.range(start)]
    }
    unsafe fn utf8(&self, start: ByteIndex) -> &'p str {
        str::from_utf8_unchecked(self.bytes(start))
    }
    fn buffer(&self) -> &'p ByteSlice {
        self.scanner.buffer
    }
    unsafe fn intern_utf8_identifier(&mut self, start: ByteIndex) -> IdentifierIndex {
        let utf8 = self.utf8(start);
        self.ast_mut().intern_identifier(utf8)
    }
    unsafe fn intern_utf8_literal(&mut self, start: ByteIndex) -> LiteralIndex {
        let utf8 = self.utf8(start);
        self.ast_mut().intern_literal(utf8)
    }

    pub fn ast(&self) -> &Ast<'a> {
        self.tokenizer.ast()
    }
    pub fn ast_mut(&mut self) -> &mut Ast<'a> {
        self.tokenizer.ast_mut()
    }

    fn utf8_syntax_error(&mut self, error: ErrorTermError, start: ByteIndex) {
        let literal = unsafe { self.intern_utf8_literal(start) };
        self.tokenizer
            .on_expression_token(ErrorTerm(error, literal), self.range(start));
    }

    fn raw_syntax_error(&mut self, error: RawErrorTermError, start: ByteIndex) {
        let bytes = self.bytes(start);
        let raw_literal = self.ast_mut().raw_literals.push(bytes.into());
        self.tokenizer
            .on_expression_token(RawErrorTerm(error, raw_literal), self.range(start));
    }

    fn integer(&mut self, start: ByteIndex) {
        self.scanner.next_while(Digit);
        if self.scanner.next_while_identifier() {
            return self.utf8_syntax_error(IdentifierStartsWithNumber, start);
        }
        let literal = unsafe { self.intern_utf8_literal(start) };
        self.tokenizer
            .on_expression_token(IntegerLiteral(literal), self.range(start))
    }

    fn identifier(&mut self, start: ByteIndex) {
        self.scanner.next_while_identifier();
        let identifier = unsafe { self.intern_utf8_identifier(start) };
        self.tokenizer
            .on_expression_token(RawIdentifier(identifier), self.range(start))
    }

    fn operator(&mut self, start: ByteIndex) {
        self.scanner.next_while(CharType::Operator);

        let term_is_about_to_end = {
            let char_type = self.scanner.peek();
            char_type.is_whitespace()
                || char_type.is_close()
                || char_type.is_separator()
                || (char_type == Colon && !self.scanner.peek_at(1).is_always_right_operand())
        };

        // If the term is about to end, this operator is postfix. i.e. "a? + 2"
        if self.tokenizer.in_term() && term_is_about_to_end {
            let operator = unsafe { self.intern_utf8_identifier(start) };
            self.tokenizer
                .on_operator_token(PostfixOperator(operator), self.range(start));
        // If we're *not* in a term, and there is something else right after the
        // operator, it is prefix. i.e. "+1"
        } else if !self.tokenizer.in_term() && !term_is_about_to_end {
            let operator = unsafe { self.intern_utf8_identifier(start) };
            self.tokenizer
                .on_expression_token(PrefixOperator(operator), self.range(start));
        // Otherwise, it's infix. i.e. "1+2" or "1 + 2"
        } else {
            let token = if Self::is_assignment_operator(self.bytes(start)) {
                let with_equal_sign = unsafe { self.utf8(start) };
                let without_equal_sign = &with_equal_sign[0..with_equal_sign.len() - 1];
                let operator = self.ast_mut().intern_identifier(without_equal_sign);
                InfixAssignment(operator)
            } else {
                let operator = unsafe { self.intern_utf8_identifier(start) };
                InfixOperator(operator)
            };
            // If the infix operator is like a+b, it's inside the term. If it's
            // like a + b, it's outside (like a separator).
            if self.tokenizer.in_term() {
                self.tokenizer.on_operator_token(token, self.range(start));
            } else {
                self.tokenizer.on_separator(token, self.range(start));
            }
        }
    }

    fn separator(&mut self, start: ByteIndex) {
        let operator = unsafe { self.intern_utf8_identifier(start) };
        self.tokenizer
            .on_separator(InfixOperator(operator), self.range(start))
    }

    // Colon is, sadly, just a little ... special.
    // If we're succeeded by an operand, and we're not in a term ("1 + :a", "a :b"), we are a prefix.
    // If we're succeeded by an operand, and we're in a term, and we're preceded by an operator ("1+:a"), we are a prefix.
    // Else, we are separator. ("a:b", a:-b", "a: b", "a:")
    // See where the "operator" function calculates whether the term is about to end for the other
    // relevant silliness to ensure "a+:b" means "(a) + (:b)".
    fn colon(&mut self, start: ByteIndex) {
        let operator = unsafe { self.intern_utf8_identifier(start) };
        if (!self.tokenizer.in_term() || self.tokenizer.prev_was_operator)
            && self.scanner.peek().is_always_right_operand()
        {
            self.tokenizer
                .on_expression_token(PrefixOperator(operator), self.range(start));
        } else {
            self.tokenizer
                .on_separator(InfixOperator(operator), self.range(start));
        }
    }

    // Anything ending with exactly one = is assignment, EXCEPT
    // >=, != and <=.
    fn is_assignment_operator(slice: &[u8]) -> bool {
        if slice[slice.len() - 1] != b'=' {
            return false;
        }
        if slice.len() < 2 {
            return true;
        }
        let prev_ch = slice[slice.len() - 2];
        if prev_ch == b'=' {
            return false;
        }
        if slice.len() > 2 {
            return true;
        }
        !matches!(prev_ch, b'!' | b'>' | b'<')
    }

    fn newline(&mut self, start: ByteIndex) {
        self.tokenizer.on_space(self.range(start));
        self.line_start();
    }

    fn line_ending(&mut self, start: ByteIndex) {
        self.store_whitespace_in_char_data(start);
        self.tokenizer.on_space(self.range(start));
        self.line_start();
    }

    fn line_start(&mut self) {
        let start = self.scanner.index;
        self.ast_mut().char_data.line_starts.push(start);

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
        self.scanner.next_while(Space);
        if self.scanner.next_while_horizontal_whitespace() {
            Some(self.store_whitespace_in_char_data(start))
        } else {
            None
        }
    }

    fn space(&mut self, start: ByteIndex) {
        self.read_space(start);
        self.tokenizer.on_space(self.range(start))
    }

    fn horizontal_whitespace(&mut self, start: ByteIndex) {
        self.scanner.next_while_horizontal_whitespace();
        self.store_whitespace_in_char_data(start);
        self.tokenizer.on_space(self.range(start))
    }

    // # <comment>
    fn comment(&mut self, start: ByteIndex) {
        self.scanner.next_until_eol();
        let bytes = self.bytes(start);
        self.ast_mut().char_data.append_comment(bytes, start);
        self.tokenizer.on_comment(self.range(start));
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
        let utf8 = unsafe { self.utf8(start) };
        self.ast_mut().char_data.append_whitespace(utf8, start)
    }
}

impl<'p> Scanner<'p> {
    fn new(buffer: &'p ByteSlice) -> Self {
        Scanner {
            buffer,
            index: 0.into(),
        }
    }
    fn next(&mut self) -> CharType {
        let (char_type, char_length) = CharType::read(self.buffer, self.index);
        if char_length == 0 {
            assert!(char_type == Eof);
        } else {
            self.advance(char_length);
        }
        char_type
    }

    fn peek(&self) -> CharType {
        CharType::peek(self.buffer, self.index)
    }

    fn peek_at<At: Into<Delta<ByteIndex>>>(&self, delta: At) -> CharType {
        CharType::peek(self.buffer, self.index + delta.into())
    }

    fn next_while(&mut self, if_type: CharType) -> bool {
        if self.next_if(if_type) {
            while self.next_if(if_type) {}
            true
        } else {
            false
        }
    }

    fn next_until_eol(&mut self) {
        loop {
            let (char_type, char_length) = CharType::read(self.buffer, self.index);
            if char_type.ends_line() {
                return;
            }
            self.advance(char_length);
        }
    }

    fn next_while_horizontal_whitespace(&mut self) -> bool {
        let mut found = false;
        loop {
            let (char_type, char_length) = CharType::read(self.buffer, self.index);
            if char_type.is_horizontal_whitespace() {
                self.advance(char_length);
                found = true;
            } else {
                break;
            }
        }
        found
    }

    fn next_while_identifier(&mut self) -> bool {
        let mut found = false;
        loop {
            let (char_type, char_length) = CharType::read(self.buffer, self.index);
            if char_type.is_identifier_middle() {
                self.advance(char_length);
                found = true;
            } else {
                break;
            }
        }
        found
    }

    fn next_if(&mut self, if_type: CharType) -> bool {
        let (char_type, char_length) = CharType::read(self.buffer, self.index);
        if char_type == if_type {
            self.advance(char_length);
            true
        } else {
            false
        }
    }

    fn advance(&mut self, char_length: Delta<ByteIndex>) {
        assert!(char_length > 0);
        self.index += char_length;
    }
}

impl CharType {
    fn read(buffer: &ByteSlice, index: ByteIndex) -> (CharType, Delta<ByteIndex>) {
        if let Some(byte_type) = ByteType::peek(buffer, index) {
            match byte_type {
                Char(char_type) => (char_type, 1.into()),
                CarriageReturn => {
                    let char_length = if let Some(&b'\n') = buffer.get(index + 1) {
                        2
                    } else {
                        1
                    };
                    (LineEnding, char_length.into())
                }
                ByteType::Utf8LeadingByte(char_length) => {
                    if Self::is_valid_utf8_char(buffer, index, char_length) {
                        (Unsupported, char_length)
                    } else {
                        (InvalidUtf8, 1.into())
                    }
                }
            }
        } else {
            (Eof, 0.into())
        }
    }

    fn peek(buffer: &ByteSlice, index: ByteIndex) -> CharType {
        CharType::read(buffer, index).0
    }

    fn is_valid_utf8_char(
        buffer: &ByteSlice,
        index: ByteIndex,
        char_length: Delta<ByteIndex>,
    ) -> bool {
        if index + char_length > buffer.len() {
            return false;
        }
        match char_length {
            Delta(ByteIndex(2)) => ByteType::is_utf8_cont(buffer[index + 1]),
            Delta(ByteIndex(3)) => {
                ByteType::is_utf8_cont(buffer[index + 1])
                    && ByteType::is_utf8_cont(buffer[index + 2])
            }
            Delta(ByteIndex(4)) => {
                ByteType::is_utf8_cont(buffer[index + 1])
                    && ByteType::is_utf8_cont(buffer[index + 2])
                    && ByteType::is_utf8_cont(buffer[index + 3])
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn is_identifier_middle(self) -> bool {
        matches!(self, Identifier | Digit)
    }

    pub(crate) fn is_whitespace(self) -> bool {
        matches!(
            self,
            Space | Newline | HorizontalWhitespace | Unsupported | InvalidUtf8 | Hash | Eof
        )
    }

    pub(crate) fn is_horizontal_whitespace(self) -> bool {
        matches!(self, Space | HorizontalWhitespace)
    }

    pub(crate) fn ends_line(self) -> bool {
        matches!(self, Newline | LineEnding | Eof)
    }

    pub(crate) fn is_close(self) -> bool {
        matches!(self, CloseParen | CloseCurly)
    }

    pub(crate) fn is_open(self) -> bool {
        matches!(self, OpenParen | OpenCurly)
    }

    pub(crate) fn is_separator(self) -> bool {
        matches!(self, Separator)
    }

    pub(crate) fn is_always_operand(self) -> bool {
        matches!(self, Digit | Identifier)
    }

    pub(crate) fn is_always_right_operand(self) -> bool {
        self.is_always_operand() || self.is_open()
    }
}

impl ByteType {
    fn peek(buffer: &ByteSlice, index: ByteIndex) -> Option<ByteType> {
        if index >= buffer.len() {
            None
        } else {
            Some(ByteType::from_byte(buffer[index]))
        }
    }

    fn from_byte(byte: u8) -> ByteType {
        match byte {
            b'+' | b'-' | b'*' | b'/' | b'=' | b'>' | b'<' | b'&' | b'|' | b'!' | b'.' => {
                Char(Operator)
            }
            b'0'..=b'9' => Char(Digit),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => Char(Identifier),
            b'(' => Char(OpenParen),
            b'{' => Char(OpenCurly),
            b')' => Char(CloseParen),
            b'}' => Char(CloseCurly),
            b';' | b',' => Char(Separator),
            b':' => Char(Colon),
            b'#' => Char(Hash),
            b' ' => Char(Space),
            b'\t' => Char(HorizontalWhitespace),
            b'\n' => Char(Newline),
            b'\r' => ByteType::CarriageReturn,
            _ => ByteType::generic(byte),
        }
    }

    fn generic(byte: u8) -> Self {
        match byte {
            0b0000_0000..=0b0111_1111 => Char(CharType::Unsupported),
            0b1100_0000..=0b1101_1111 => Utf8LeadingByte(Delta(ByteIndex(2))),
            0b1110_0000..=0b1110_1111 => Utf8LeadingByte(Delta(ByteIndex(3))),
            0b1111_0000..=0b1111_0111 => Utf8LeadingByte(Delta(ByteIndex(4))),
            _ => Char(CharType::InvalidUtf8),
        }
    }

    fn is_utf8_cont(byte: u8) -> bool {
        (0b1000_0000..0b1011_1111).contains(&byte)
    }
}
