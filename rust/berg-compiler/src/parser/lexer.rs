use public::*;
use parser::Parser;

use std::ops::Range;
use std::ops::RangeInclusive;

fn match_all<'c, 'b>(
    range: &RangeInclusive<u8>,
    parser: &mut Parser<'c, 'b>,
    mut index: ByteIndex,
) -> ByteIndex {
    while index < parser.buffer.len() && range.contains(parser.buffer[index]) {
        index += 1;
    }
    index
}

pub fn term<'c, 'b>(parser: &mut Parser<'c, 'b>) -> bool {
    let mut index = parser.index;
    if index >= parser.buffer.len() {
        return false;
    }

    match parser.buffer[index] {
        b'0'...b'9' => {
            index = match_all(&(b'0'..=b'9'), parser, index + 1);
            token(IntegerLiteral, parser, index)
        }
        _ => invalid_or_unsupported(parser, index),
    }
}

pub fn invalid_or_unsupported<'c, 'b>(parser: &mut Parser<'c, 'b>, mut index: ByteIndex) -> bool {
    match parser.buffer[index] {
        0x00..UTF8_CONT_START => {
            return error(UnsupportedCharacters, parser, index + 1);
        },
        UTF8_2_START..UTF8_3_START => if parser.buffer.len() > index + 1
            && UTF8_CONT.contains(parser.buffer[index + 1])
        {
            return error(UnsupportedCharacters, parser, index + 2);
        },
        UTF8_3_START..UTF8_4_START => if parser.buffer.len() > index + 2
            && UTF8_CONT.contains(parser.buffer[index + 1])
            && UTF8_CONT.contains(parser.buffer[index + 2])
        {
            return error(UnsupportedCharacters, parser, index + 3);
        },
        UTF8_4_START..UTF8_INVALID_START => if parser.buffer.len() > index + 3
            && UTF8_CONT.contains(parser.buffer[index + 1])
            && UTF8_CONT.contains(parser.buffer[index + 2])
            && UTF8_CONT.contains(parser.buffer[index + 3])
        {
            return error(UnsupportedCharacters, parser, index + 4);
        },
        _ => {}
    }
    index += 1;
    while index < parser.buffer.len() {
        match parser.buffer[index] {
            UTF8_CONT_START..UTF8_2_START | UTF8_INVALID_START..0xFF => {
                index += 1;
            }
            _ => break,
        }
    }
    invalid_utf8_error(InvalidUtf8, parser, index)
}

fn invalid_utf8_error<'c, 'b>(
    error_type: CompileErrorType,
    parser: &mut Parser<'c, 'b>,
    end: ByteIndex,
) -> bool {
    let start = parser.index;
    parser.index = end;
    let string = &parser.buffer[start..end];
    let error = error_type.invalid(parser.source, start, string);
    parser.report(error);
    true
}

fn error<'c, 'b>(
    error_type: CompileErrorType,
    parser: &mut Parser<'c, 'b>,
    end: ByteIndex,
) -> bool {
    let start = parser.index;
    parser.index = end;
    let buf = parser.buffer[start..end].to_vec();
    let string = unsafe { String::from_utf8_unchecked(buf) };
    let error = error_type.at(parser.source, start, &string);
    parser.report(error);
    true
}

fn token<'c, 'b>(
    expression_type: SyntaxExpressionType,
    parser: &mut Parser<'c, 'b>,
    end: ByteIndex,
) -> bool {
    let start = parser.index;
    parser.index = end;
    let buf = parser.buffer[start..end].to_vec();
    let string = unsafe { String::from_utf8_unchecked(buf) };
    parser
        .expressions
        .push(SyntaxExpression::new(expression_type, start, string));
    true
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
