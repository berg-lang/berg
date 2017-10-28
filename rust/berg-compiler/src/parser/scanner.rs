use public::*;

use compiler::compile_errors::SourceCompileErrors;
use parser::char_data::ByteIndex;
use std::str;
use parser::scanner::ByteType::*;
use parser::scanner::CharType::*;

/// Parses a source file (bytes) into tokens (integers, operators, etc.).
#[derive(Debug,Copy,Clone)]
pub(crate) struct Scanner { index: ByteIndex }

#[derive(Debug,Copy,Clone)]
pub enum TermResult {
    IntegerLiteral(ByteIndex),
    Prefix(ByteIndex),
    MissingTerm,
}

#[derive(Debug,Copy,Clone)]
pub enum OperatorResult {
    Infix(ByteIndex),
    Postfix(ByteIndex),
    MissingInfix,
    Eof,
}

impl Default for Scanner {
    fn default() -> Self { Scanner { index: ByteIndex(0) } }
}

impl Scanner {
    pub fn index(&self) -> ByteIndex { self.index }

    pub fn next_term(
        &mut self,
        buffer: &[u8],
        errors: &mut SourceCompileErrors,
    ) -> TermResult {
        while self.index < buffer.len() {
            let byte_type = ByteType::from(buffer[usize::from(self.index)]);
            match byte_type {
                Char(Digit) => return TermResult::IntegerLiteral(self.read_many(buffer, 1, Digit)),
                Char(Operator) => return TermResult::Prefix(self.read_many(buffer, 1, Operator)),
                InvalidUtf8|Char(Unsupported)|Utf8LeadingByte(_) => self.report_unsupported_or_invalid_utf8(buffer, byte_type, errors),
            }
        }

        // Handle EOF
        TermResult::MissingTerm
    }

    pub fn next_operator(
        &mut self,
        buffer: &[u8],
        errors: &mut SourceCompileErrors,
    ) -> OperatorResult {
        while self.index < buffer.len() {
            let byte_type = ByteType::from(buffer[usize::from(self.index)]);
            match byte_type {
                Char(Digit) => return OperatorResult::MissingInfix,
                Char(Operator) => {
                    let start = self.read_many(buffer, 1, Operator);
                    if self.next_char_has_left_operand(buffer) {
                        return OperatorResult::Postfix(start);
                    } else {
                        return OperatorResult::Infix(start);
                    }
                },
                InvalidUtf8|Char(Unsupported)|Utf8LeadingByte(_) => self.report_unsupported_or_invalid_utf8(buffer, byte_type, errors),
            }
        }

        OperatorResult::Eof
    }

    fn report_unsupported_or_invalid_utf8(
        &mut self,
        buffer: &[u8],
        byte_type: ByteType,
        errors: &mut SourceCompileErrors
    ) {
        match byte_type {
            Char(Unsupported) => self.report_unsupported(buffer, 1, errors),
            InvalidUtf8 => self.report_invalid_utf8(buffer, errors),
            Utf8LeadingByte(char_length) => {
                if self.is_valid_utf8_char(buffer, char_length) {
                    self.report_unsupported(buffer, char_length, errors)
                } else {
                    self.report_invalid_utf8(buffer, errors)
                }
            },
            Char(_) => unreachable!(),
        }
    }

    fn report_unsupported(
        &mut self,
        buffer: &[u8],
        char_length: u8,
        errors: &mut SourceCompileErrors
    ) {
        let start = self.read_many(buffer, char_length, CharType::Unsupported);
        let range = start..self.index;
        let bytes = &buffer[usize::from(start)..usize::from(self.index)];
        let string = unsafe { str::from_utf8_unchecked(bytes) };
        errors.report_at(CompileErrorType::UnsupportedCharacters, range, string);
    }

    fn report_invalid_utf8(
        &mut self,
        buffer: &[u8],
        errors: &mut SourceCompileErrors
    ) {
        let start = self.index;
        self.index += 1;
        while self.index < buffer.len() && ByteType::from(buffer[usize::from(self.index)]) == ByteType::InvalidUtf8 {
            self.index += 1;
        }
        let range = start..self.index;
        let bytes = &buffer[usize::from(start)..usize::from(self.index)];
        errors.report_invalid_utf8(range, bytes);
    }

    // #[inline(always)]
    fn read_many(
        &mut self,
        buffer: &[u8],
        char_length: u8,
        if_char_type: CharType
    ) -> ByteIndex {
        let start = self.index;
        self.index += usize::from(char_length);
        while self.index < buffer.len() {
            match self.peek_char(buffer) {
                Some((char_type, char_length)) if char_type == if_char_type => self.index += usize::from(char_length),
                _ => break,
            }
        }
        start
    }

    fn next_char_has_left_operand(
        &mut self,
        buffer: &[u8],
    ) -> bool {
        if self.index < buffer.len() {
            match self.peek_char(buffer) {
                Some((CharType::Digit, _)) => false,
                _ => true,
            }
        } else {
            true
        }
    }

    // #[inline(always)]
    fn peek_char(
        &mut self,
        buffer: &[u8]
    ) -> Option<(CharType, u8)> {
        match ByteType::from(buffer[usize::from(self.index)]) {
            ByteType::Char(char_type) => Some((char_type, 1)),
            ByteType::InvalidUtf8 => None,
            ByteType::Utf8LeadingByte(n) => self.peek_char_utf8_leading(buffer, n),
        }
    }

    fn peek_char_utf8_leading(
        &mut self,
        buffer: &[u8],
        char_length: u8
    ) -> Option<(CharType, u8)> {
        if self.is_valid_utf8_char(buffer, char_length) {
            Some((CharType::Unsupported, char_length))
        } else {
            None
        }
    }

    fn is_valid_utf8_char(
        &self,
        buffer: &[u8],
        char_length: u8
    ) -> bool {
        let index = usize::from(self.index);
        if index + usize::from(char_length) > buffer.len() {
            return false;
        }
        match char_length {
            2 => ByteType::is_utf8_cont(buffer[index+1]),
            3 => ByteType::is_utf8_cont(buffer[index+1]) && ByteType::is_utf8_cont(buffer[index+2]),
            4 => ByteType::is_utf8_cont(buffer[index+1]) && ByteType::is_utf8_cont(buffer[index+2]) && ByteType::is_utf8_cont(buffer[index+3]),
            _ => unreachable!()
        }
    }
}


#[derive(Debug,Copy,Clone,PartialEq)]
enum CharType {
    Digit,
    Operator,
    Unsupported,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum ByteType {
    Char(CharType),
    InvalidUtf8,
    Utf8LeadingByte(u8),
}

impl From<u8> for ByteType {
    fn from(byte: u8) -> Self {
        use parser::scanner::ByteType::*;
        match byte {
            b'+'|b'-'|b'*'|b'/' => Char(CharType::Operator),
            b'0'...b'9' => Char(CharType::Digit),
            _ => ByteType::from_generic(byte)
        }
    }
}

impl ByteType {
    fn from_generic(byte: u8) -> Self {
        use parser::scanner::ByteType::*;
        match byte {
            0b0000_0000...0b0111_1111 => Char(CharType::Unsupported),
            0b1100_0000...0b1101_1111 => Utf8LeadingByte(2),
            0b1110_0000...0b1110_1111 => Utf8LeadingByte(3),
            0b1111_0000...0b1111_0111 => Utf8LeadingByte(4),
            _ => InvalidUtf8,
        }
    }

    fn is_utf8_cont(byte: u8) -> bool {
        byte >= 0b1000_0000 && byte < 0b1011_1111
    }
}
