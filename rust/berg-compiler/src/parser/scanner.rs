use indexed_vec::Delta;
use compiler::source_data::{ByteSlice,ByteIndex};
use parser::scanner::ByteType::*;
use parser::scanner::CharType::*;

#[derive(Default)]
pub(super) struct Scanner {
    pub(super) index: ByteIndex,
}

impl Scanner {
    pub(super) fn next(&mut self, buffer: &ByteSlice) -> Option<CharType> {
        if let Some((char_type, char_length)) = CharType::read(buffer, self.index) {
            self.index += char_length;
            Some(char_type)
        } else {
            None
        }
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
            match CharType::read(buffer, self.index) {
                Some((Identifier,char_length))|Some((Digit,char_length)) => {
                    self.index += char_length;
                    found = true;
                },
                _ => break,
            }
        }
        found
    }

    pub(super) fn next_if(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
        if let Some((char_type, char_length)) = CharType::read(buffer, self.index) {
            if char_type == if_type {
                self.index += char_length;
                return true;
            }
        }
        false
    }

    pub(super) fn peek_if_space(&self, buffer: &ByteSlice) -> bool {
        if let Some(char_type) = CharType::peek(buffer, self.index) {
            match char_type {
                Space|Newline|Unsupported|InvalidUtf8 => true,
                Open|Close|Operator|Separator|Digit|Identifier|Colon => false,
            }
        } else {
            true
        }
    }

    pub(super) fn peek_if_space_or_operator(&self, buffer: &ByteSlice) -> bool {
        if let Some(char_type) = CharType::peek(buffer, self.index) {
            match char_type {
                Close|Operator|Separator|Space|Newline|Unsupported|InvalidUtf8 => true,
                Open|Digit|Identifier|Colon => false,
            }
        } else {
            true
        }
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub(super) enum CharType {
    Digit,
    Identifier,
    Operator,
    Open,
    Close,
    Separator,
    Space,
    Colon,
    Newline,
    Unsupported,
    InvalidUtf8,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum ByteType {
    Char(CharType),
    CarriageReturn,
    Utf8LeadingByte(Delta<ByteIndex>),
}

impl CharType {
    fn read(buffer: &ByteSlice, index: ByteIndex) -> Option<(CharType,Delta<ByteIndex>)> {
        if index >= buffer.len() {
            return None;
        }
        match ByteType::from_byte(buffer[index]) {
            ByteType::Char(char_type) => Some((char_type,1.into())),
            ByteType::CarriageReturn => {
                let char_length = if let Some(&b'\n') = buffer.get(index+1) { 2 } else { 1 };
                Some((CharType::Newline, char_length.into()))
            },
            ByteType::Utf8LeadingByte(char_length) => {
                if Self::is_valid_utf8_char(buffer, index, char_length) {
                    Some((Unsupported, char_length))
                } else {
                    Some((InvalidUtf8, 1.into()))
                }
            },
        }
    }

    fn peek(buffer: &ByteSlice, index: ByteIndex) -> Option<CharType> {
        if let Some((char_type, _)) = Self::read(buffer, index) {
            Some(char_type)
        } else {
            None
        }
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
}

impl ByteType {
    fn from_byte(byte: u8) -> ByteType {
        match byte {
            b'+'|b'-'|b'*'|b'/'|b'='|b'>'|b'<'|b'&'|b'|'|b'!' => Char(Operator),
            b'0'...b'9' => Char(Digit),
            b'a'...b'z'|b'A'...b'Z'|b'_' => Char(Identifier),
            b'(' => Char(Open),
            b')' => Char(Close),
            b':' => Char(Colon),
            b';' => Char(Separator),
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
