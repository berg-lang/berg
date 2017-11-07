use indexed_vec::Delta;
use compiler::source_data::{ByteSlice,ByteIndex};
use parser::scanner::ByteType::*;
use parser::scanner::CharType::*;
use std::str;

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
        Self::read_if(buffer, index, |_| true)
    }

    pub(super) fn read_if<IfChar: Fn(CharType)->bool>(buffer: &ByteSlice, index: &mut ByteIndex, if_char: IfChar) -> Option<CharType> {
        Self::peek_map(buffer, *index, |char_type,char_length| {
            if char_type.is_some() && if_char(char_type.unwrap()) {
                *index += char_length;
                char_type
            } else {
                None
            }
        })
    }

    pub(super) fn read_many(self, buffer: &ByteSlice, index: &mut ByteIndex) {
        while Self::read_if(buffer, index, |char_type| char_type == self).is_some() {}
    }

    pub(super) fn read_many_to_str<'b>(self, buffer: &'b ByteSlice, start: ByteIndex, index: &mut ByteIndex) -> &'b str {
        assert_ne!(self, CharType::InvalidUtf8);
        self.read_many(buffer, index);
        unsafe { str::from_utf8_unchecked(&buffer[start..*index]) }
    }

    // pub(super) fn peek(buffer: &ByteSlice, index: ByteIndex) -> Option<CharType> {
    //     Self::peek_map(buffer, index, |char_type,_| char_type)
    // }

    pub(super) fn peek_if<IfChar: Fn(Option<CharType>)->bool>(buffer: &ByteSlice, index: ByteIndex, if_char: IfChar) -> bool {
        Self::peek_map(buffer, index, |char_type,_| if_char(char_type))
    }

    pub(super) fn peek_map<T, MapChar: FnMut(Option<CharType>,Delta<ByteIndex>)->T>(buffer: &ByteSlice, index: ByteIndex, mut map_char: MapChar) -> T {
        if index >= buffer.len() {
            return map_char(None, Delta(ByteIndex(0)));
        }
        let byte_type = ByteType::from_byte(buffer[index]);
        match byte_type {
            ByteType::Char(char_type) => map_char(Some(char_type),Delta(ByteIndex(1))),
            ByteType::Utf8LeadingByte(char_length) => {
                if Self::is_valid_utf8_char(buffer, index, char_length) {
                    map_char(Some(Unsupported), char_length)
                } else {
                    map_char(Some(InvalidUtf8), Delta(ByteIndex(1)))
                }
            },
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
            b'+'|b'-'|b'*'|b'/' => Char(Operator),
            b'0'...b'9' => Char(Digit),
            b'(' => Char(Open),
            b')' => Char(Close),
            b' '|b'\t' => Char(Space),
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
