use public::*;

use parser::char_data::CharData;
use parser::token_pool::*;
use std::ops::Index;
use std::ops::Range;
use std::str;

// Wrapper so that it will have ByteIndex sized indexes
#[derive(Debug)]
pub(crate) struct Scanner<'s, 'c: 's> {
    pub compiler: &'s Compiler<'c>,
    pub source: SourceIndex,
    pub index: ByteIndex,
    pub char_data: CharData,
    pub token_pool: TokenPool,
    buffer: &'s [u8],
}

impl<'s, 'c: 's> Scanner<'s, 'c> {
    pub fn new(compiler: &'s Compiler<'c>, source: SourceIndex, mut buffer: &'s [u8]) -> Self {
        // NOTE you can have a buffer 4G in size and *may* have more than 4G tokens if there are zero-width tokens.
        // We don't as of the time of this writing, but I have been considering it.
        if buffer.len() > ByteIndex::MAX.into() {
            compiler.report_source_only(SourceTooLarge, source);
            buffer = &buffer[0..ByteIndex::MAX.into()]
        }
        let char_data = Default::default();
        let token_pool = Default::default();
        let index = Default::default();
        Scanner {
            compiler,
            source,
            buffer,
            char_data,
            token_pool,
            index,
        }
    }
    pub fn len(&self) -> ByteIndex {
        self.buffer.len().into()
    }
    pub fn eof(&self) -> bool {
        self.index >= self.len()
    }

    pub fn match_all<Matcher: Fn(u8) -> bool>(&mut self, matches: Matcher) -> Option<ByteIndex> {
        if matches(self[self.index]) {
            let mut index = self.index + 1;
            while index < self.len() && matches(self[index]) {
                index += 1;
            }
            Some(index)
        } else {
            None
        }
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

    pub fn take_token(&mut self, end: ByteIndex) -> (Range<ByteIndex>, TokenIndex) {
        let start = self.index;
        self.index = end;
        let string = unsafe { str::from_utf8_unchecked(&self.buffer[start.into()..end.into()]) };
        let token = self.token_pool.intern(string);
        (start..end, token)
    }

    pub fn take_string(&mut self, end: ByteIndex) -> (Range<ByteIndex>, String) {
        let start = self.index;
        let vec = self[start..end].to_vec();
        let string = unsafe { String::from_utf8_unchecked(vec) };
        self.index = end;
        (start..end, string)
    }

    fn is_utf8_cont(byte: u8) -> bool {
        byte >= UTF8_CONT_START && byte <= UTF8_2_START
    }

    // If the next character is a UTF-8 codepoint, returns its length
    fn valid_utf8_char_length(&self) -> usize {
        let index = self.index;
        let ch = self[index];
        if ch < UTF8_CONT_START {
            1
        } else if ch >= UTF8_2_START && ch < UTF8_3_START {
            if self.len() > index + 1 && Self::is_utf8_cont(self[index + 1]) {
                2
            } else {
                0
            }
        } else if ch >= UTF8_3_START && ch < UTF8_4_START {
            if self.len() > index + 2
                && Self::is_utf8_cont(self[index + 1])
                && Self::is_utf8_cont(self[index + 2])
            {
                3
            } else {
                0
            }
        } else if ch >= UTF8_4_START && ch < UTF8_INVALID_START {
            if self.len() > index + 3
                && Self::is_utf8_cont(self[index + 1])
                && Self::is_utf8_cont(self[index + 2])
                && Self::is_utf8_cont(self[index + 3])
            {
                4
            } else {
                0
            }
        } else {
            0            
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

impl<'s, 'c: 's> Index<ByteIndex> for Scanner<'s, 'c> {
    type Output = u8;
    fn index(&self, index: ByteIndex) -> &u8 {
        &self.buffer[usize::from(index)]
    }
}
impl<'s, 'c: 's> Index<Range<ByteIndex>> for Scanner<'s, 'c> {
    type Output = [u8];
    fn index(&self, range: Range<ByteIndex>) -> &[u8] {
        let start = range.start.into();
        let end = range.end.into();
        let range = Range { start, end };
        &self.buffer[range]
    }
}
