use public::*;

use std::ops::Index;
use std::ops::Range;
use std::ops::RangeInclusive;
use std::str;

// Wrapper so that it will have ByteIndex sized indexes
#[derive(Debug)]
pub struct Scanner<'s, 'c: 's> {
    pub compiler: &'s Compiler<'c>,
    pub source: SourceIndex,
    pub index: ByteIndex,
    buffer: &'s [u8],
}

impl<'s, 'c: 's> Scanner<'s, 'c> {
    pub fn new(compiler: &'s Compiler<'c>, source: SourceIndex, buffer: &'s [u8]) -> Self {
        Scanner { compiler, source, buffer, index: 0 }
        // TODO add "SourceTooBig" error for > 4G
    }
    pub fn len(&self) -> ByteIndex {
        self.buffer.len() as ByteIndex
    }

    pub fn match_all(
        &mut self,
        range: &RangeInclusive<u8>,
        mut index: ByteIndex,
    ) -> ByteIndex {
        while index < self.len() && range.contains(self[index]) {
            index += 1;
        }
        index
    }

    pub fn is_valid_char(&self) -> bool {
        self.valid_utf8_char_length() > 0
    }

    pub fn take_valid_char(&mut self, string: &mut String) -> bool {
        let len = self.valid_utf8_char_length();
        if len > 0 {
            let start = self.index;
            self.index += len;
            let bytes = &self[start..self.index];
            string.push_str(unsafe { str::from_utf8_unchecked(bytes) });
            true
        } else {
            false
        }
    }

    pub fn take_byte(&mut self, bytes: &mut Vec<u8>) {
        bytes.push(self[self.index]);
        self.index += 1
    }

    pub fn take_string(&mut self, end: ByteIndex) -> (ByteIndex, String) {
        let start = self.index;
        self.index = end;
        let vec = self[start..self.index].to_vec();
        let string = unsafe { String::from_utf8_unchecked(vec) };
        (start, string)
    }

    pub fn take_token(&mut self, expression_type: SyntaxExpressionType, end: ByteIndex) -> SyntaxExpression {
        let (start, string) = self.take_string(end);
        SyntaxExpression::new(expression_type, start, string)
    }

    // If the next character is a UTF-8 codepoint, returns its length
    fn valid_utf8_char_length(&self) -> ByteIndex {
        let index = self.index;
        match self[self.index] {
            0x00..UTF8_CONT_START => {
                1
            },
            UTF8_2_START..UTF8_3_START => {
                if self.len() > index + 1
                    && UTF8_CONT.contains(self[index + 1])
                {
                    2
                } else {
                    0
                }
            },
            UTF8_3_START..UTF8_4_START => {
                if self.len() > index + 2
                    && UTF8_CONT.contains(self[index + 1])
                    && UTF8_CONT.contains(self[index + 2])
                {
                    3
                } else {
                    0
                }
            },
            UTF8_4_START..UTF8_INVALID_START => {
                if self.len() > index + 3
                    && UTF8_CONT.contains(self[index + 1])
                    && UTF8_CONT.contains(self[index + 2])
                    && UTF8_CONT.contains(self[index + 3])
                {
                    4
                } else {
                    0
                }
            },
            _ => {
                return 0;
            }
        }
    }
}

// Start of a UTF-8 continuation byte
const UTF8_CONT_START: u8 = 0b1000_0000;
// Start of a UTF-8 2-byte leading byte
const UTF8_2_START: u8 = 0b1100_0000;
// Start of a UTF-8 3-byte leading byte
const UTF8_3_START: u8 = 0b1110_0000;
// Start of a UTF-8 3-byte leading byte
const UTF8_4_START: u8 = 0b1111_0000;
// Invalid UTF-8 bytes from here to 256. Can never occur.
const UTF8_INVALID_START: u8 = 0b1111_1000;

// const ASCII: Range<u8> = 0x00..UTF8_CONT_START;
const UTF8_CONT: Range<u8> = UTF8_CONT_START..UTF8_2_START;
// const UTF8_2: Range<u8> = UTF8_2_START..UTF8_3_START;
// const UTF8_3: Range<u8> = UTF8_3_START..UTF8_4_START;
// const UTF8_4: Range<u8> = UTF8_4_START..UTF8_INVALID_START;
// const UTF8_INVALID: Range<u8> = UTF8_4_START..0xFF;

impl<'s, 'c: 's> Index<ByteIndex> for Scanner<'s, 'c> {
    type Output = u8;
    fn index(&self, index: ByteIndex) -> &u8 {
        &self.buffer[index as usize]
    }
}
impl<'s, 'c: 's> Index<Range<ByteIndex>> for Scanner<'s, 'c> {
    type Output = [u8];
    fn index(&self, range: Range<ByteIndex>) -> &[u8] {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        &self.buffer[range]
    }
}
impl<'s, 'c: 's> Index<RangeInclusive<ByteIndex>> for Scanner<'s, 'c> {
    type Output = [u8];
    fn index(&self, range: RangeInclusive<ByteIndex>) -> &[u8] {
        let range = Range {
            start: range.start as usize,
            end: range.end as usize,
        };
        &self.buffer[range]
    }
}

