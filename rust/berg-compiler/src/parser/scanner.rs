use compiler::source_data::ByteIndex;
use parser::scanner::ByteType::*;
use parser::scanner::CharType::*;

#[derive(Debug,Copy,Clone)]
pub enum Symbol {
    Integer,
    Operator,
    Open,
    Close,
    UnsupportedCharacters,
    InvalidUtf8Bytes,
}

pub fn next(buffer: &[u8], mut index: ByteIndex) -> Option<(Symbol, ByteIndex)> {
    if usize::from(index) >= buffer.len() {
        return None;
    }

    let byte_type = ByteType::from(buffer[usize::from(index)]);
    index += 1;
    let symbol = match byte_type {
        Char(Digit) => { read_bytes_while(buffer, &mut index, byte_type); Symbol::Integer },
        Char(Operator) => { read_bytes_while(buffer, &mut index, byte_type); Symbol::Operator },
        Char(Open) => Symbol::Open,
        Char(Close) => Symbol::Close,
        InvalidUtf8|Char(Unsupported)|Utf8LeadingByte(_) => { unsupported_or_invalid_utf8(buffer, &mut index, byte_type) },
    };
    Some((symbol, index))
}

pub fn next_has_left_operand(
    buffer: &[u8],
    index: ByteIndex
) -> bool {
    if index < buffer.len() {
        match peek_char(buffer, index) {
            Some((CharType::Digit, _)) => false,
            _ => true,
        }
    } else {
        true
    }
}

fn unsupported_or_invalid_utf8(
    buffer: &[u8],
    index: &mut ByteIndex,
    byte_type: ByteType,
) -> Symbol {
    match byte_type {
        Char(Unsupported) => { read_many_unsupported(buffer, index); Symbol::UnsupportedCharacters },
        InvalidUtf8 => { read_many_invalid_utf8(buffer, index); Symbol::InvalidUtf8Bytes },
        Utf8LeadingByte(char_length) => {
            if is_valid_utf8_char(buffer, *index, char_length) {
                *index += usize::from(char_length-1);
                read_many_unsupported(buffer, index);
                Symbol::UnsupportedCharacters
            } else {
                read_many_invalid_utf8(buffer, index);
                Symbol::InvalidUtf8Bytes
            }
        },
        Char(_) => unreachable!(),
    }
}

fn read_bytes_while(buffer: &[u8], index: &mut ByteIndex, byte_type: ByteType) {
    while usize::from(*index) < buffer.len() && ByteType::from(buffer[usize::from(*index)]) == byte_type {
        *index += 1;
    }
}

fn read_many_unsupported(buffer: &[u8], index: &mut ByteIndex) {
    while usize::from(*index) < buffer.len() {
        if let Some((Unsupported, char_length)) = peek_char(buffer, *index) {
            *index += usize::from(char_length);
        } else {
            break;
        }
    }
}

fn read_many_invalid_utf8(buffer: &[u8], index: &mut ByteIndex) {
    while usize::from(*index) < buffer.len() && peek_char(buffer, *index).is_none() {
        *index += 1;
    }
}

// #[inline(always)]
fn peek_char(buffer: &[u8], index: ByteIndex) -> Option<(CharType, u8)> {
    match ByteType::from(buffer[usize::from(index)]) {
        ByteType::Char(char_type) => Some((char_type, 1)),
        ByteType::InvalidUtf8 => None,
        ByteType::Utf8LeadingByte(n) => peek_char_utf8_leading(buffer, index, n),
    }
}

fn peek_char_utf8_leading(buffer: &[u8], index: ByteIndex, char_length: u8) -> Option<(CharType, u8)> {
    if is_valid_utf8_char(buffer, index, char_length) {
        Some((CharType::Unsupported, char_length))
    } else {
        None
    }
}

fn is_valid_utf8_char(buffer: &[u8], index: ByteIndex, char_length: u8) -> bool {
    let index = usize::from(index);
    if index + usize::from(char_length) > buffer.len() {
        return false;
    }
    match char_length {
        2 => ByteType::is_utf8_cont(buffer[index+1]),
        3 => ByteType::is_utf8_cont(buffer[index+1]) && ByteType::is_utf8_cont(buffer[index+2]),
        4 => ByteType::is_utf8_cont(buffer[index+1]) && ByteType::is_utf8_cont(buffer[index+2]) && ByteType::is_utf8_cont(buffer[index+3]),
        _ => unreachable!()
    }
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum CharType {
    Digit,
    Operator,
    Open,
    Close,
    Unsupported,
}

#[derive(Debug,Copy,Clone,PartialEq)]
enum ByteType {
    Char(CharType),
    InvalidUtf8,
    Utf8LeadingByte(u8),
}

impl From<u8> for ByteType {
    fn from(byte: u8) -> Self {
        match byte {
            b'+'|b'-'|b'*'|b'/' => Char(Operator),
            b'0'...b'9' => Char(Digit),
            b'(' => Char(Open),
            b')' => Char(Close),
            _ => ByteType::from_generic(byte)
        }
    }
}

impl ByteType {
    fn from_generic(byte: u8) -> Self {
        use parser::scanner::ByteType::*;
        match byte {
            0b0000_0000...0b0111_1111 => Char(CharType::Unsupported),
            0b1100_0000...0b1101_1111 => Utf8LeadingByte(2),
            0b1110_0000...0b1110_1111 => Utf8LeadingByte(3),
            0b1111_0000...0b1111_0111 => Utf8LeadingByte(4),
            _ => InvalidUtf8,
        }
    }

    fn is_utf8_cont(byte: u8) -> bool {
        byte >= 0b1000_0000 && byte < 0b1011_1111
    }
}
