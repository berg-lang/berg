use source::compile_errors::CompileErrorCode;
use ast::IdentifierIndex;
use ast::token::Token::*;
use ast::token::ExpressionBoundary::*;
use ast::intern_pool::Pool;
use source::parse_result::{ByteIndex,ByteSlice,ParseResult};
use util::indexed_vec::Delta;
use parser::sequencer::ByteType::*;
use parser::sequencer::CharType::*;
use parser::tokenizer::Tokenizer;
use std::str;

// Chunks up the source into sequences: space, newlines, operators, etc.
// Passes these to the Tokenizer to handle expression and whitespace rules.
#[derive(Debug)]
pub(super) struct Sequencer {
    tokenizer: Tokenizer,
}

impl Sequencer {
    pub(super) fn new() -> Self {
        Sequencer {
            tokenizer: Tokenizer::new()
        }
    }

    pub(super) fn parse(&mut self, buffer: &ByteSlice, parse_result: &mut ParseResult) {
        let mut scanner = Scanner::default();
        let mut start = scanner.index;

        self.tokenizer.on_source_start(start, parse_result);

        loop {
            let char_type = scanner.next(buffer);
            println!("CHAR TYPE #{:?}", char_type);

            match char_type {
                Digit       => self.integer(buffer, start, &mut scanner, parse_result),
                Identifier  => self.identifier(buffer, start, &mut scanner, parse_result),
                Operator    => self.operator(buffer, start, &mut scanner, parse_result),
                Separator   => self.separator(buffer, start, &mut scanner, parse_result),
                Colon       => self.colon(buffer, start, &mut scanner, parse_result),
                OpenParen   => self.tokenizer.on_open(Parentheses, start..scanner.index, parse_result),
                CloseParen  => self.tokenizer.on_close(Parentheses, start..scanner.index, parse_result),
                OpenCurly   => self.tokenizer.on_open(CurlyBraces, start..scanner.index, parse_result),
                CloseCurly  => self.tokenizer.on_close(CurlyBraces, start..scanner.index, parse_result),
                Newline     => self.newline(buffer, start, &scanner, parse_result),
                Space       => self.space(buffer, start, &mut scanner, parse_result),
                Unsupported => self.unsupported(buffer, start, &mut scanner, parse_result),
                InvalidUtf8 => self.invalid_utf8(buffer, start, &mut scanner, parse_result),
                Eof         => break,
            };
         
            start = scanner.index;
        }

        assert!(start == scanner.index);
        assert!(scanner.index == buffer.len()); 

        self.tokenizer.on_source_end(scanner.index, parse_result)
    }

    fn syntax_error(&mut self, error: CompileErrorCode, start: ByteIndex, scanner: &Scanner, parse_result: &mut ParseResult) {
        let range = start..scanner.index;
        self.tokenizer.on_term_token(ErrorTerm(error), range, parse_result);
    }

    fn integer(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        scanner.next_while(Digit, buffer);
        if scanner.next_while_identifier(buffer) {
            return self.syntax_error(CompileErrorCode::IdentifierStartsWithNumber, start, scanner, parse_result);
        }
        let range = start..scanner.index;
        let string = unsafe { str::from_utf8_unchecked(&buffer[&range]) };
        let literal = parse_result.literals.add(string);
        self.tokenizer.on_term_token(IntegerLiteral(literal), range, parse_result)
    }

    fn identifier(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        scanner.next_while_identifier(buffer);
        let range = start..scanner.index;
        let string = unsafe { str::from_utf8_unchecked(&buffer[&range]) };
        let identifier = parse_result.identifiers.add(string);
        self.tokenizer.on_term_token(RawIdentifier(identifier), range, parse_result)
    }

    fn make_identifier(&mut self, slice: &[u8], parse_result: &mut ParseResult) -> IdentifierIndex {
        let string = unsafe { str::from_utf8_unchecked(slice) };
        parse_result.identifiers.add(string)
    }

    fn operator(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        scanner.next_while(CharType::Operator, buffer);

        let term_is_about_to_end = {
            let char_type = scanner.peek(buffer);
            char_type.is_space() || char_type.is_close() || char_type.is_separator() ||
            (char_type == Colon && !scanner.peek_at(buffer, 1).is_right_term_operand())
        };

        let range = start..scanner.index;
        if self.tokenizer.in_term && term_is_about_to_end {
            let identifier = self.make_identifier(&buffer[&range], parse_result);
            self.tokenizer.on_term_token(PostfixOperator(identifier), range, parse_result);
        } else if !self.tokenizer.in_term && !term_is_about_to_end {
            let identifier = self.make_identifier(&buffer[&range], parse_result);
            self.tokenizer.on_term_token(PrefixOperator(identifier), range, parse_result);
        } else {
            let token = if Self::is_assignment_operator(&buffer[&range]) {
                InfixAssignment(self.make_identifier(&buffer[start..scanner.index-1], parse_result))
            } else {
                InfixOperator(self.make_identifier(&buffer[&range], parse_result))
            };
            // If the infix operator is like a+b, it's inside the term. If it's
            // like a + b, it's outside (like a separator).
            if self.tokenizer.in_term {
                self.tokenizer.on_term_token(token, range, parse_result);
            } else {
                self.tokenizer.on_separator(token, range, parse_result);
            }
        }
    }

    fn separator(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        let string = self.make_identifier(&buffer[start..scanner.index], parse_result);
        self.tokenizer.on_separator(InfixOperator(string), start..scanner.index, parse_result)
    }

    // Colon is, sadly, just a little ... special.
    // If we're succeeded by an operand, and we're not in a term ("1 + :a", "a :b"), we are a prefix.
    // If we're succeeded by an operand, and we're in a term, and we're preceded by an operator ("1+:a"), we are a prefix.
    // Else, we are separator. ("a:b", a:-b", "a: b", "a:")
    // See where the "operator" function calculates whether the term is about to end for the other
    // relevant silliness to ensure "a+:b" means "(a) + (:b)".
    fn colon(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        let range = start..scanner.index;
        let identifier = self.make_identifier(&buffer[&range], parse_result);
        if (!self.tokenizer.in_term || self.tokenizer.operator) && scanner.peek(buffer).is_right_term_operand() {
            self.tokenizer.on_term_token(PrefixOperator(identifier), range, parse_result);
        } else {
            self.tokenizer.on_separator(InfixOperator(identifier), range, parse_result);
        }
    }

    // Anything ending with exactly one = is assignment, EXCEPT
    // >=, != and <=.
    fn is_assignment_operator(slice: &[u8]) -> bool {
        if slice[slice.len()-1] != b'=' { return false; }
        if slice.len() < 2 { return true; }
        let prev_ch = slice[slice.len()-2];
        if prev_ch == b'=' { return false; }
        if slice.len() > 2 { return true; }
        match prev_ch { b'!'|b'>'|b'<' => false, _ => true }
    }

    fn newline(&mut self, _: &ByteSlice, start: ByteIndex, scanner: &Scanner, parse_result: &mut ParseResult) {
        parse_result.char_data.append_line(scanner.index);
        self.tokenizer.on_newline(start, ((scanner.index - start).0).0 as u8, parse_result)
    }

    fn space(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        scanner.next_while(Space, buffer);
        self.tokenizer.on_space(start, parse_result)
    }

    fn unsupported(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        scanner.next_while(Unsupported, buffer);
        self.syntax_error(CompileErrorCode::UnsupportedCharacters, start, scanner, parse_result)
    }
 
    fn invalid_utf8(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner, parse_result: &mut ParseResult) {
        scanner.next_while(InvalidUtf8, buffer);
        self.syntax_error(CompileErrorCode::InvalidUtf8, start, scanner, parse_result)
    }
}

#[derive(Default,Clone)]
struct Scanner {
    index: ByteIndex,
}

impl Scanner {
    fn next(&mut self, buffer: &ByteSlice) -> CharType {
        let (char_type, char_length) = CharType::read(buffer, self.index);
        if char_length == 0 {
            assert!(char_type == Eof);
        } else {
            self.advance(char_length);
        }
        char_type
    }

    fn peek(&self, buffer: &ByteSlice) -> CharType {
        CharType::peek(buffer, self.index)
    }

    fn peek_at<At: Into<Delta<ByteIndex>>>(&self, buffer: &ByteSlice, delta: At) -> CharType {
        CharType::peek(buffer, self.index+delta.into())
    }

    fn next_while(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
        if self.next_if(if_type, buffer) {
            while self.next_if(if_type, buffer) {}
            true
        } else {
            false
        }
    }

    fn next_while_identifier(&mut self, buffer: &ByteSlice) -> bool {
        let mut found = false;
        loop {
            let (char_type, char_length) = CharType::read(buffer, self.index);
            if char_type.is_identifier_middle() {
                self.advance(char_length);
                found = true;
            } else {
                break;
            }
        }
        found
    }

    fn next_if(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
        let (char_type, char_length) = CharType::read(buffer, self.index);
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

#[derive(Debug,Copy,Clone,PartialEq)]
pub(super) enum CharType {
    Digit,
    Identifier,
    Operator,
    OpenParen,
    CloseParen,
    OpenCurly,
    CloseCurly,
    Separator,
    Colon,
    Space,
    Newline,
    Unsupported,
    InvalidUtf8,
    Eof,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum ByteType {
    Char(CharType),
    CarriageReturn,
    Utf8LeadingByte(Delta<ByteIndex>),
}

impl CharType {
    fn read(buffer: &ByteSlice, index: ByteIndex) -> (CharType,Delta<ByteIndex>) {
        if let Some(byte_type) = ByteType::peek(buffer, index) {
            match byte_type {
                Char(char_type) => (char_type,1.into()),
                CarriageReturn => {
                    let char_length = if let Some(&b'\n') = buffer.get(index+1) { 2 } else { 1 };
                    (Newline, char_length.into())
                },
                ByteType::Utf8LeadingByte(char_length) => {
                    if Self::is_valid_utf8_char(buffer, index, char_length) {
                        (Unsupported, char_length)
                    } else {
                        (InvalidUtf8, 1.into())
                    }
                },
            }
        } else {
            (Eof,0.into())
        }
    }

    fn peek(buffer: &ByteSlice, index: ByteIndex) -> CharType {
        CharType::read(buffer, index).0
    }

    fn is_valid_utf8_char(buffer: &ByteSlice, index: ByteIndex, char_length: Delta<ByteIndex>) -> bool {
        if index + char_length > buffer.len() {
            return false;
        }
        match char_length {
            Delta(ByteIndex(2)) => ByteType::is_utf8_cont(buffer[index+1]),
            Delta(ByteIndex(3)) => ByteType::is_utf8_cont(buffer[index+1]) && ByteType::is_utf8_cont(buffer[index+2]),
            Delta(ByteIndex(4)) => ByteType::is_utf8_cont(buffer[index+1]) && ByteType::is_utf8_cont(buffer[index+2]) && ByteType::is_utf8_cont(buffer[index+3]),
            _ => unreachable!()
        }
    }

    pub(crate) fn is_identifier_middle(self) -> bool {
        match self {
            Identifier|Digit => true,
            _ => false,
        }
    }

    pub(crate) fn is_space(self) -> bool {
        match self {
            Space|Newline|Unsupported|InvalidUtf8|Eof => true,
            _ => false,
        }
    }

    pub(crate) fn is_close(self) -> bool {
        match self {
            CloseParen|CloseCurly => true,
            _ => false,
        }
    }

    pub(crate) fn is_open(self) -> bool {
        match self {
            OpenParen|OpenCurly => true,
            _ => false,
        }
    }

    pub(crate) fn is_separator(self) -> bool {
        match self {
            Separator => true,
            _ => false,
        }
    }

    pub(crate) fn is_always_operand(self) -> bool {
        match self {
            Digit|Identifier => true,
            _ => false,
        }
    }

    pub(crate) fn is_right_term_operand(self) -> bool {
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
            b'+'|b'-'|b'*'|b'/'|b'='|b'>'|b'<'|b'&'|b'|'|b'!' => Char(Operator),
            b'0'...b'9' => Char(Digit),
            b'a'...b'z'|b'A'...b'Z'|b'_' => Char(Identifier),
            b'(' => Char(OpenParen),
            b'{' => Char(OpenCurly),
            b')' => Char(CloseParen),
            b'}' => Char(CloseCurly),
            b';' => Char(Separator),
            b':' => Char(Colon),
            b' '|b'\t' => Char(Space),
            b'\n' => Char(Newline),
            b'\r' => ByteType::CarriageReturn,
            _ => ByteType::from_generic(byte)
        }
    }

    fn from_generic(byte: u8) -> Self {
        match byte {
            0b0000_0000...0b0111_1111 => Char(CharType::Unsupported),
            0b1100_0000...0b1101_1111 => Utf8LeadingByte(Delta(ByteIndex(2))),
            0b1110_0000...0b1110_1111 => Utf8LeadingByte(Delta(ByteIndex(3))),
            0b1111_0000...0b1111_0111 => Utf8LeadingByte(Delta(ByteIndex(4))),
            _ => Char(CharType::InvalidUtf8),
        }
    }

    fn is_utf8_cont(byte: u8) -> bool {
        byte >= 0b1000_0000 && byte < 0b1011_1111
    }
}
