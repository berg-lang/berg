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
    match parser.buffer[index] {
        b'0'...b'9' => {
            index = match_all(&(b'0'..=b'9'), parser, index + 1);
            token(IntegerLiteral, parser, index)
        },
        _ => false,
    }
}

pub fn advance<'c, 'b, F: Fn(&mut Parser<'c, 'b>)->bool>(parser: &mut Parser<'c, 'b>, supported: F) -> bool {
    if parser.index >= parser.buffer.len() {
        false
    } else if supported(parser) {
        true
    } else {
        invalid_or_unsupported(parser, supported)
    }
}

pub fn invalid_or_unsupported<'c, 'b, F: Fn(&mut Parser<'c, 'b>)->bool>(parser: &mut Parser<'c, 'b>, supported: F) -> bool {
    let valid_length = valid_utf8_char_length(parser, parser.index);
    if valid_length > 0 {
        // It's a valid, but unsupported character. Now we are going to have to loop until we hit a supported (or invalid) character.
        let start = parser.index;
        let mut end = start + valid_length;
        parser.index = end;
        while parser.index < parser.buffer.len() && !supported(parser) {
            let valid_length = valid_utf8_char_length(parser, parser.index);
            if valid_length == 0 {
                // Invalid. Let's report our unsupported character error and let the next function call take that on.
                break;
            }
            // Unsupported. Skip this character and move on to the next.
            end += valid_length;
            parser.index = end;
        }
        report_raw_error_at(UnsupportedCharacters, parser, start, end)
    } else {
        // Invalid UTF-8. Read invalid characters until you find something valid.
        let mut index = parser.index + 1;
        while index < parser.buffer.len() && valid_utf8_char_length(parser, index) == 0 {
            index += 1;
        }
        invalid_utf8_error(InvalidUtf8, parser, index)
    }
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

// If the next character is a UTF-8 codepoint, returns its length
pub fn valid_utf8_char_length<'c, 'b>(parser: &Parser<'c, 'b>, index: ByteIndex) -> ByteIndex {
    match parser.buffer[index] {
        0x00..UTF8_CONT_START => {
            1
        },
        UTF8_2_START..UTF8_3_START => {
            if parser.buffer.len() > index + 1
                && UTF8_CONT.contains(parser.buffer[index + 1])
            {
                2
            } else {
                0
            }
        },
        UTF8_3_START..UTF8_4_START => {
            if parser.buffer.len() > index + 2
                && UTF8_CONT.contains(parser.buffer[index + 1])
                && UTF8_CONT.contains(parser.buffer[index + 2])
            {
                3
            } else {
                0
            }
        },
        UTF8_4_START..UTF8_INVALID_START => {
            if parser.buffer.len() > index + 3
                && UTF8_CONT.contains(parser.buffer[index + 1])
                && UTF8_CONT.contains(parser.buffer[index + 2])
                && UTF8_CONT.contains(parser.buffer[index + 3])
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

// fn error<'c, 'b>(
//     error_type: CompileErrorType,
//     parser: &mut Parser<'c, 'b>,
//     end: ByteIndex,
// ) -> bool {
//     let start = parser.index;
//     parser.index = end;
//     report_raw_error_at(error_type, parser, start, end)
// }

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

// An error that started before the current parser position.
fn report_raw_error_at<'c, 'b>(
    error_type: CompileErrorType,
    parser: &mut Parser<'c, 'b>,
    start: ByteIndex,
    end: ByteIndex,
) -> bool {
    let buf = parser.buffer[start..end].to_vec();
    let string = unsafe { String::from_utf8_unchecked(buf) };
    let error = error_type.at(parser.source, start, &string);
    parser.report(error);
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
