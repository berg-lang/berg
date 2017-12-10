use indexed_vec::Delta;
use compiler::source_data::{ByteSlice,ByteIndex};
use parser::scanner::ByteType::*;
use parser::scanner::CharType::*;

#[derive(Default,Clone)]
pub(super) struct Scanner {
    pub(super) index: ByteIndex,
}

impl Scanner {
    pub(super) fn next(&mut self, buffer: &ByteSlice) -> CharType {
        let (char_type, char_length) = CharType::read(buffer, self.index);
        if char_length == 0 {
            assert!(char_type == Eof);
        } else {
            self.advance(char_length);
        }
        char_type
    }

    pub(super) fn peek(&self, buffer: &ByteSlice) -> CharType {
        CharType::peek(buffer, self.index)
    }

    pub(super) fn peek_at<At: Into<Delta<ByteIndex>>>(&self, buffer: &ByteSlice, delta: At) -> CharType {
        CharType::peek(buffer, self.index+delta.into())
    }

    pub(super) fn next_while(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
        if self.next_if(if_type, buffer) {
            while self.next_if(if_type, buffer) {}
            true
        } else {
            false
        }
    }

    pub(super) fn next_while_identifier(&mut self, buffer: &ByteSlice) -> bool {
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

    pub(super) fn next_if(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
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
        use parser::scanner::ByteType::*;
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
