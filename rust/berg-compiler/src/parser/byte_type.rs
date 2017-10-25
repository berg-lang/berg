use public::*;

#[derive(Debug,Copy,Clone)]
pub(crate) struct ByteScanner {
    index: ByteIndex,
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub(crate) enum ByteType {
    Operator,
    Digit,
    Generic(Utf8ByteType),
}

#[derive(Debug,Copy,Clone,PartialEq)]
pub(crate) enum Utf8ByteType {
    UnsupportedAscii,
    Utf8Cont,
    Utf8Lead2,
    Utf8Lead3,
    Utf8Lead4,
    InvalidUtf8,
}

impl From<u8> for ByteType {
    fn from(byte: u8) -> Self {
        use parser::byte_type::ByteType::*;
        match byte {
            b'+'|b'-'|b'*'|b'/' => Operator,
            b'0'...b'9' => Digit,
            _ => Generic(Utf8ByteType::from(byte)),
        }
    }
}

impl From<u8> for Utf8ByteType {
    // Because we don't expect these to happen much, keep them off the inlined hot path.
    fn from(byte: u8) -> Self {
        use parser::byte_type::Utf8ByteType::*;
        match byte {
            0b0000_0000...0b0111_1111 => UnsupportedAscii,
            0b1000_0000...0b1011_1111 => Utf8Cont,
            0b1100_0000...0b1101_1111 => Utf8Lead2,
            0b1110_0000...0b1110_1111 => Utf8Lead3,
            0b1111_0000...0b1111_0111 => Utf8Lead4,
            0b1111_1000...0b1111_1111 => InvalidUtf8,
            _ => unreachable!(), // TODO am I wrong? Why do I have to specify this?
        }
    }
}
