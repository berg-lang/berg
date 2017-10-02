use compiler::*;
use parser::*;
use parser::results::*;
use parser::stream_buffer::*;
use parser::scanner::*;

use std::ops::Range;

pub struct Grammar;

impl<'s, 'p: 's, 'c: 'p> Grammar {
    pub fn term<Buf: StreamBuffer>(parser: &'s mut Parser<'p, 'c, Buf>) -> bool {
        let mut accept = Scanner::<'s, 'p, 'c, Buf>::new(parser);
        if let Ok(ch) = accept.accept_byte() {
            match ch {
                b'0'...b'9' => {
                    accept.many(b'0'..=b'9');
                    accept.term(IntegerLiteral);
                },
                b'\n' => accept.mark_newline(),
                b'\r' => {
                    accept.one(b'\n');
                    accept.mark_newline()
                },
                _ => Self::unsupported_or_invalid(ch, accept),
            }
            true
        } else {
            false
        }
    }

    pub fn unsupported_or_invalid<Buf: StreamBuffer>(byte: u8, mut accept: Scanner<'s, 'p, 'c, Buf>) {
        match byte {
            // ASCII
            0..UTF8_CONT_START    => Self::unsupported_character(accept),
            // 2-byte UTF-8
            UTF8_2_START..UTF8_3_START   => {
                if accept.one(UTF8_CONT) {
                    Self::unsupported_character(accept)
                } else {
                    Self::invalid_utf8(accept)
                }
            },
            // 3-byte UTF-8
            UTF8_3_START..UTF8_4_START   => {
                if accept.exactly(2, UTF8_CONT) {
                    Self::unsupported_character(accept)
                } else {
                    Self::invalid_utf8(accept)
                }
            },
            // 4-byte UTF-8
            UTF8_4_START..UTF8_INVALID_START   => {
                if accept.exactly(3, UTF8_CONT) {
                    Self::unsupported_character(accept)
                } else {
                    Self::invalid_utf8(accept)
                }
            },
            // Invalid UTF-8
            _ => Self::invalid_utf8(accept),
        }
    }

    fn unsupported_character<Buf: StreamBuffer>(accept: Scanner<'s, 'p, 'c, Buf>) {
        accept.error(UnsupportedCharacters);
    }

    fn invalid_utf8<Buf: StreamBuffer>(mut accept: Scanner<'s, 'p, 'c, Buf>) {
        accept.many((UTF8_CONT, UTF8_INVALID));
        accept.error_invalid(InvalidUtf8)
    }
}


// Start of a UTF-8 continuation byte
const UTF8_CONT_START: u8    = 0b10000000;
// Start of a UTF-8 2-byte leading byte
const UTF8_2_START: u8       = 0b11000000;
// Start of a UTF-8 3-byte leading byte
const UTF8_3_START: u8       = 0b11100000;
// Start of a UTF-8 3-byte leading byte
const UTF8_4_START: u8       = 0b11110000;
// Invalid UTF-8 bytes from here to 256. Can never occur.
const UTF8_INVALID_START: u8 = 0b11111000;

// const ASCII: Range<u8> = 0x00..UTF8_CONT_START;
const UTF8_CONT: Range<u8> = UTF8_CONT_START..UTF8_2_START;
// const UTF8_2: Range<u8> = UTF8_2_START..UTF8_3_START;
// const UTF8_3: Range<u8> = UTF8_3_START..UTF8_4_START;
// const UTF8_4: Range<u8> = UTF8_4_START..UTF8_INVALID_START;
const UTF8_INVALID: Range<u8> = UTF8_4_START..0xFF;

