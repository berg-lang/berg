use std::borrow::Cow;

use crate::syntax::{ByteIndex, ByteSlice};
use berg_util::Delta;
use ByteType::*;
use CharType::*;

///
/// Scans UTF-8 identifying characters.
///
#[derive(Debug)]
pub struct Scanner {
    /// The buffer we're scanning.
    pub buffer: Cow<'static, ByteSlice>,
    /// The index of the next byte to read from the buffer.
    pub index: ByteIndex,
}

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

impl Scanner {
    pub fn new(buffer: Cow<'static, ByteSlice>) -> Self {
        Scanner {
            buffer,
            index: 0.into(),
        }
    }

    pub fn buffer(&self) -> &ByteSlice {
        &self.buffer
    }

    pub fn next(&mut self) -> CharType {
        let (char_type, char_length) = CharType::read(self.buffer(), self.index);
        if char_length == 0 {
            assert!(char_type == CharType::Eof);
        } else {
            self.advance(char_length);
        }
        char_type
    }

    pub fn peek(&self) -> CharType {
        CharType::peek(self.buffer(), self.index)
    }

    pub fn peek_at<At: Into<Delta<ByteIndex>>>(&self, delta: At) -> CharType {
        CharType::peek(self.buffer(), self.index + delta.into())
    }

    pub fn next_while(&mut self, if_type: CharType) -> bool {
        if self.next_if(if_type) {
            while self.next_if(if_type) {}
            true
        } else {
            false
        }
    }

    pub fn next_until_eol(&mut self) {
        loop {
            let (char_type, char_length) = CharType::read(self.buffer(), self.index);
            if char_type.ends_line() {
                return;
            }
            self.advance(char_length);
        }
    }

    pub fn next_while_horizontal_whitespace(&mut self) -> bool {
        let mut found = false;
        loop {
            let (char_type, char_length) = CharType::read(self.buffer(), self.index);
            if char_type.is_horizontal_whitespace() {
                self.advance(char_length);
                found = true;
            } else {
                break;
            }
        }
        found
    }

    pub fn next_while_identifier(&mut self) -> bool {
        let mut found = false;
        loop {
            let (char_type, char_length) = CharType::read(self.buffer(), self.index);
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
        let (char_type, char_length) = CharType::read(self.buffer(), self.index);
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

    pub(crate) fn at_end(&self) -> bool {
        self.index == self.buffer().len()
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
