use crate::*;

#[test]
fn unsupported() {
    expect("`").to_error(UnsupportedCharacters, "`")
}
#[test]
fn unsupported_multibyte() {
    expect("⌂").to_error(UnsupportedCharacters, "⌂")
}
#[test]
fn unsupported_multiple() {
    expect("⌂`⌂").to_error(UnsupportedCharacters, "⌂`⌂")
}
#[test]
fn unsupported_then_ok() {
    expect("`1").to_error(UnsupportedCharacters, "`")
}
#[test]
fn unsupported_multibyte_then_ok() {
    expect("⌂1").to_error(UnsupportedCharacters, "⌂")
}
#[test]
fn unsupported_multiple_then_ok() {
    expect("⌂`⌂1").to_error(UnsupportedCharacters, "⌂`⌂")
}
#[test]
fn ok_then_unsupported() {
    expect("1`").to_error(UnsupportedOperator, 1..1)
}
#[test]
fn ok_then_unsupported_multibyte() {
    expect("1⌂").to_error(UnsupportedOperator, 1..1)
}
#[test]
fn ok_then_unsupported_multiple() {
    expect("1⌂`⌂").to_error(UnsupportedOperator, 1..1)
}

#[test]
fn invalid_utf8_no_leading_byte() {
    expect(&[0b1000_0000]).to_error(InvalidUtf8, 0)
}
#[test]
fn invalid_utf8_invalid_byte() {
    expect(&[0b1111_1000]).to_error(InvalidUtf8, 0)
}
#[test]
fn invalid_utf8_multiple() {
    expect(&[0b1000_0000, 0b1111_1000, 0b1000_0000]).to_error(InvalidUtf8, 0..=2)
}
#[test]
fn invalid_utf8_multiple_then_ok() {
    expect(&[0b1000_0000, 0b1111_1000, 0b1000_0000, b'1']).to_error(InvalidUtf8, 0..=2)
}

#[test]
fn invalid_utf8_too_small_2() {
    expect(&[0b1100_0000, b'1']).to_error(InvalidUtf8, 0)
}
#[test]
fn invalid_utf8_too_small_eof_2() {
    expect(&[0b1100_0000]).to_error(InvalidUtf8, 0)
}
#[test]
fn invalid_utf8_too_small_3() {
    expect(&[0b1110_0000, 0b1000_0000, b'1']).to_error(InvalidUtf8, 0..=1)
}
#[test]
fn invalid_utf8_too_small_eof_3() {
    expect(&[0b1110_0000, 0b1000_0000]).to_error(InvalidUtf8, 0..=1)
}
#[test]
fn invalid_utf8_too_small_4() {
    expect(&[0b1110_0000, 0b1000_0000, b'1']).to_error(InvalidUtf8, 0..=1)
}
#[test]
fn invalid_utf8_too_small_eof_4() {
    expect(&[0b1111_0000, 0b1000_0000, 0b1000_0000]).to_error(InvalidUtf8, 0..=2)
}

#[test]
fn unsupported_then_invalid() {
    expect(&[b'`', 0b1000_0000]).to_error(UnsupportedCharacters, 0)
}
#[test]
fn unsupported_then_invalid_multibyte() {
    // 0xE28C82 is ⌂
    expect(&[0xE2,0x8C,0x82, 0b1000_0000]).to_error(UnsupportedCharacters, 0..=2)
}
#[test]
fn unsupported_then_invalid_multiple() {
    // 0xE28C82 is ⌂
    expect(&[0xE2,0x8C,0x82, b'`', 0xE2,0x8C,0x82, 0b1000_0000, 0b1000_0000]).to_error(UnsupportedCharacters, 0..=6)
}
#[test]
fn invalid_then_unsupported() {
    expect(&[0b1000_0000, b'`']).to_error(InvalidUtf8, 0)
}
#[test]
fn invalid_then_unsupported_multibyte() {
    // 0xE28C82 is ⌂
    expect(&[0b1000_0000, 0xE2,0x8C,0x82]).to_error(InvalidUtf8, 0)
}
#[test]
fn invalid_then_unsupported_multiple() {
    // 0xE28C82 is ⌂
    expect(&[0b1000_0000, 0b1000_0000, 0xE2,0x8C,0x82, b'`', 0xE2,0x8C,0x82]).to_error(InvalidUtf8, 0..=1)
}
