use parser::char_data::ByteIndex;
use parser::byte_type::*;
use parser::char_type::CharType::*;

#[derive(Debug,Copy,Clone,PartialEq)]
pub(crate) enum CharType {
    Operator,
    Digit,
    InvalidUtf8,
    Unsupported,
}

impl CharType {
    pub fn next(buffer: &[u8], index: ByteIndex) -> Option<(CharType, ByteIndex)> {
        if index >= buffer.len() {
            return None;
        }
        let char_type = match ByteType::from(buffer[usize::from(index)]) {
            ByteType::Operator         => Operator,
            ByteType::Digit            => Digit,
            ByteType::Generic(byte_type) => {
                let result = Self::next_unsupported_or_invalid_utf8(byte_type, buffer, index+1);
                return Some(result);
            }
        };
        Some((char_type, index+1))
    }

    pub fn next_while(buffer: &[u8], mut index: ByteIndex, expected: CharType) -> ByteIndex {
        while let Some((actual, next_index)) = Self::next(buffer, index) {
            if expected == actual {
                index = next_index;
            } else {
                break;
            }
        }
        index
    }

    fn next_unsupported_or_invalid_utf8(byte_type: Utf8ByteType, buffer: &[u8], index: ByteIndex) -> (CharType, ByteIndex) {
        use parser::byte_type::Utf8ByteType::*;
        match byte_type {
            UnsupportedAscii => { return (CharType::Unsupported, index) },
            Utf8Lead2 => {
                if index < buffer.len() &&
                    Utf8ByteType::from(buffer[usize::from(index)]) == Utf8Cont {
                    return (CharType::Unsupported, index+1);
                }
            },
            Utf8Lead3 => {
                if index+1 < buffer.len() &&
                    Utf8ByteType::from(buffer[usize::from(index)]) == Utf8Cont &&
                    Utf8ByteType::from(buffer[usize::from(index)+1]) == Utf8Cont {
                    return (CharType::Unsupported, index+2);
                }
            },
            Utf8Lead4 => {
                if index+2 < buffer.len() &&
                    Utf8ByteType::from(buffer[usize::from(index)]) == Utf8Cont &&
                    Utf8ByteType::from(buffer[usize::from(index)+1]) == Utf8Cont &&
                    Utf8ByteType::from(buffer[usize::from(index)+2]) == Utf8Cont {
                    return (CharType::Unsupported, index+3);
                }
            },
            InvalidUtf8|Utf8Cont => {},
        }
        (CharType::InvalidUtf8, index)
    }
}
