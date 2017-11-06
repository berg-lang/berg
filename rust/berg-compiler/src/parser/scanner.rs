use indexed_vec::Delta;
use compiler::source_data::{ByteSlice,ByteIndex};
use parser::scanner::ByteType::*;
use parser::scanner::CharType::*;

#[derive(Debug,Copy,Clone,PartialEq)]
pub(super) enum CharType {
    Digit,
    Operator,
    Open,
    Close,
    Space,
    Unsupported,
    InvalidUtf8,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum ByteType {
    Char(CharType),
    Utf8LeadingByte(Delta<ByteIndex>),
}

impl CharType {
    pub(super) fn read(buffer: &ByteSlice, index: &mut ByteIndex) -> Option<CharType> {
        if *index >= buffer.len() {
            return None;
        }
        match ByteType::from(buffer[*index]) {
            ByteType::Char(char_type) => { println!("{:?} at {}", char_type, *index); *index += 1; Some(char_type) },
            ByteType::Utf8LeadingByte(char_length) => if Self::is_valid_utf8_char(buffer, *index, char_length) {
                *index += char_length;
                Some(Unsupported)
            } else {
                *index += 1;
                Some(InvalidUtf8)
            }
        }
    }
    pub(super) fn read_many(&self, buffer: &ByteSlice, index: &mut ByteIndex) -> (ByteIndex, Option<CharType>) {
        let mut end = *index;
        while let Some(next_char) = Self::read(buffer, index) {
            if next_char != *self {
                return (end, Some(next_char));
            }
            end = *index;
        }
        (end, None)
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

impl From<u8> for ByteType {
    fn from(byte: u8) -> Self {
        match byte {
            b'+'|b'-'|b'*'|b'/' => Char(Operator),
            b'0'...b'9' => Char(Digit),
            b'(' => Char(Open),
            b')' => Char(Close),
            b' '|b'\t' => Char(Space),
            _ => ByteType::from_generic(byte)
        }
    }
}

impl ByteType {
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
