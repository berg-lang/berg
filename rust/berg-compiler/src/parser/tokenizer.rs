use public::*;

use parser::LiteralIndex;
use indexed_vec::IndexedVec;
use parser::token_pool::TokenPool;
use compiler::compile_errors::SourceCompileErrors;
use parser::IdentifierIndex;
use parser::token::*;
use parser::char_data::ByteIndex;
use parser::tokenizer::Tokenizer::*;
use std::str;

/// Parses a source file (bytes) into tokens (integers, operators, etc.).
#[derive(Debug)]
pub(crate) enum Tokenizer {
    Term,
    Operator,
}

impl Default for Tokenizer {
    fn default() -> Self { Tokenizer::Term }
}

impl Tokenizer {
    pub fn next(
        &mut self,
        buffer: &[u8],
        index: &mut ByteIndex,
        errors: &mut SourceCompileErrors,
        identifier_pool: &mut TokenPool<IdentifierIndex>,
        literal_strings: &mut IndexedVec<String,LiteralIndex>
    ) -> Option<(ByteIndex, Token)> {
        use parser::tokenizer::ByteType::*;
        use parser::tokenizer::Tokenizer::*;
        while *index < buffer.len() {
            let result = match ByteType::from(buffer[usize::from(*index)]) {
                Char(char_type) => self.next_from_char(buffer, index, char_type, 1, (errors, identifier_pool, literal_strings)),
                InvalidUtf8 => self.report_invalid_utf8(buffer, index, errors),
                Utf8LeadingByte(n) => self.next_utf8_leading(buffer, index, n, errors),
            };
            if result.is_some() {
                return result;
            }
        }

        // Handle EOF
        match *self {
            Operator => None,
            Term => self.missing_term(index),
        }
    }

    fn literal<F: Fn(LiteralIndex) -> Token>(
        &mut self,
        buffer: &[u8],
        start: ByteIndex,
        index: &mut ByteIndex,
        literal_strings: &mut IndexedVec<String,LiteralIndex>,
        to_token: F
    ) -> Option<(ByteIndex, Token)> {
        let literal = literal_strings.len();
        let string = unsafe { str::from_utf8_unchecked(&buffer[usize::from(start)..usize::from(*index)]) };
        literal_strings.push(string.to_string());
        let token = to_token(literal);
        *self = Operator;
        Some((start, token))
    }

    fn prefix(
        &mut self,
        buffer: &[u8],
        start: ByteIndex,
        index: &mut ByteIndex,
        identifier_pool: &mut TokenPool<IdentifierIndex>
    ) -> Option<(ByteIndex, Token)> {
        let identifier = unsafe { identifier_pool.intern_unchecked(&buffer[usize::from(start)..usize::from(*index)]) };
        let token = Token::Prefix(identifier);
        *self = Term;
        Some((start, token))
    }

    fn postfix(
        &mut self,
        buffer: &[u8],
        start: ByteIndex,
        index: &mut ByteIndex,
        identifier_pool: &mut TokenPool<IdentifierIndex>
    ) -> Option<(ByteIndex, Token)> {
        let identifier = unsafe { identifier_pool.intern_unchecked(&buffer[usize::from(start)..usize::from(*index)]) };
        let token = Token::Postfix(identifier);
        *self = Operator;
        Some((start, token))
    }

    fn infix(
        &mut self,
        buffer: &[u8],
        start: ByteIndex,
        index: &ByteIndex,
        identifier_pool: &mut TokenPool<IdentifierIndex>,
    ) -> Option<(ByteIndex, Token)> {
        let identifier = unsafe { identifier_pool.intern_unchecked(&buffer[usize::from(start)..usize::from(*index)]) };
        let token = Token::Infix(identifier);
        *self = Term;
        Some((start, token))
    }

    fn missing_term(&mut self, index: &ByteIndex) -> Option<(ByteIndex, Token)> {
        *self = Operator;
        Some((*index, Token::MissingTerm))
    }

    fn missing_infix(&mut self, index: &ByteIndex) -> Option<(ByteIndex, Token)> {
        *self = Operator;
        Some((*index, Token::MissingTerm))
    }

    // #[inline(always)]
    fn next_from_char(
        &mut self,
        buffer: &[u8],
        index: &mut ByteIndex,
        char_type: CharType,
        char_length: u8,
        (errors, identifier_pool, literal_strings): (&mut SourceCompileErrors, &mut TokenPool<IdentifierIndex>, &mut IndexedVec<String,LiteralIndex>)
    ) -> Option<(ByteIndex, Token)> {
        match char_type {
            CharType::Digit => match *self {
                Operator => self.missing_infix(index),
                Term => {
                    *self = Operator;
                    let start = *index;
                    CharType::read_many(buffer, index, char_length, char_type);
                    self.literal(buffer, start, index, literal_strings, Token::IntegerLiteral)
                },
            },

            CharType::Operator => {
                let start = *index;
                CharType::read_many(buffer, index, char_length, char_type);
                match *self {
                    Operator => {
                        if CharType::next_char_has_left_operand(buffer, index) {
                            self.postfix(buffer, start, index, identifier_pool)
                        } else {
                            self.infix(buffer, start, index, identifier_pool)
                        }
                    },
                    Term => self.prefix(buffer, start, index, identifier_pool)
                }
            },

            CharType::Unsupported => self.report_unsupported(buffer, index, char_length, errors),
        }
    }

    fn next_utf8_leading(
        &mut self,
        buffer: &[u8],
        index: &mut ByteIndex,
        char_length: u8,
        errors: &mut SourceCompileErrors
    ) -> Option<(ByteIndex, Token)> {
        if CharType::is_valid_utf8_char(buffer, index, char_length) {
            self.report_unsupported(buffer, index, char_length, errors)
        } else {
            self.report_invalid_utf8(buffer, index, errors)
        }
    }

    fn report_unsupported(
        &mut self,
        buffer: &[u8],
        index: &mut ByteIndex,
        char_length: u8,
        errors: &mut SourceCompileErrors
    ) -> Option<(ByteIndex, Token)> {
        let start = *index;
        CharType::read_many(buffer, index, char_length, CharType::Unsupported);
        let range = start..*index;
        let bytes = &buffer[usize::from(start)..usize::from(*index)];
        let string = unsafe { str::from_utf8_unchecked(bytes) };
        errors.report_at(CompileErrorType::UnsupportedCharacters, range, string);
        *self = Term;
        None
    }

    fn report_invalid_utf8(
        &mut self,
        buffer: &[u8],
        index: &mut ByteIndex,
        errors: &mut SourceCompileErrors
    ) -> Option<(ByteIndex, Token)> {
        let start = *index;
        *index += 1;
        while *index < buffer.len() && ByteType::from(buffer[usize::from(*index)]) == ByteType::InvalidUtf8 {
            *index += 1;
        }
        let range = start..*index;
        let bytes = &buffer[usize::from(start)..usize::from(*index)];
        errors.report_invalid_utf8(range, bytes);
        *self = Term;
        None
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum CharType {
    Digit,
    Operator,
    Unsupported,
}

impl CharType {
    // #[inline(always)]
    fn read_many(
        buffer: &[u8],
        index: &mut ByteIndex,
        char_length: u8,
        if_char_type: CharType
    ) {
        *index += usize::from(char_length);
        while *index < buffer.len() {
            match Self::peek_char(buffer, index) {
                Some((char_type, char_length)) if char_type == if_char_type => *index += usize::from(char_length),
                _ => break,
            }
        }
    }

    fn next_char_has_left_operand(
        buffer: &[u8],
        index: &ByteIndex
    ) -> bool {
        if *index < buffer.len() {
            match Self::peek_char(buffer, index) {
                Some((CharType::Digit, _)) => false,
                _ => true,
            }
        } else {
            true
        }
    }

    // #[inline(always)]
    fn peek_char(
        buffer: &[u8],
        index: &ByteIndex
    ) -> Option<(CharType, u8)> {
        match ByteType::from(buffer[usize::from(*index)]) {
            ByteType::Char(char_type) => Some((char_type, 1)),
            ByteType::InvalidUtf8 => None,
            ByteType::Utf8LeadingByte(n) => Self::peek_char_utf8_leading(buffer, index, n),
        }
    }

    fn peek_char_utf8_leading(
        buffer: &[u8],
        index: &ByteIndex,
        char_length: u8
    ) -> Option<(CharType, u8)> {
        if Self::is_valid_utf8_char(buffer, index, char_length) {
            Some((CharType::Unsupported, char_length))
        } else {
            None
        }
    }

    fn is_valid_utf8_char(
        buffer: &[u8],
        index: &ByteIndex,
        char_length: u8
    ) -> bool {
        let index = usize::from(*index);
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
enum ByteType {
    Char(CharType),
    InvalidUtf8,
    Utf8LeadingByte(u8),
}

impl From<u8> for ByteType {
    fn from(byte: u8) -> Self {
        use parser::tokenizer::ByteType::*;
        match byte {
            b'+'|b'-'|b'*'|b'/' => Char(CharType::Operator),
            b'0'...b'9' => Char(CharType::Digit),
            _ => ByteType::from_generic(byte)
        }
    }
}

impl ByteType {
    fn from_generic(byte: u8) -> Self {
        use parser::tokenizer::ByteType::*;
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
