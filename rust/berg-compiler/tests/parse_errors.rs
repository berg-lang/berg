pub mod compiler_test;
use compiler_test::*;

#[test]
fn unsupported_characters()           { expect( "`"                                      ).to_error(UnsupportedCharacters,0) }
#[test]
fn unsupported_characters_multiple()  { expect( "``"                                     ).to_error(UnsupportedCharacters,0..=1) }
#[test]
fn unsupported_characters_then_ok()   { expect( "`1"                                     ).to_error(UnsupportedCharacters,0) }
#[test]
fn unsupported_characters_multiple_then_ok() { expect( "``1"                                    ).to_error(UnsupportedCharacters,0..=1) }

#[test]
fn invalid_utf8_no_leading_byte()     { expect( &[0b1000_0000]                           ).to_error(InvalidUtf8,0) }
#[test]
fn invalid_utf8_invalid_byte()        { expect( &[0b1111_1000]                           ).to_error(InvalidUtf8,0) }
#[test]
fn invalid_utf8_multiple()            { expect( &[0b1000_0000, 0b1111_1000, 0b1000_0000] ).to_error(InvalidUtf8,0..=2) }
#[test]
fn invalid_utf8_multiple_then_ok()    { expect( &[0b1000_0000, 0b1111_1000, 0b1000_0000, b'1'] ).to_error(InvalidUtf8,0..=2) }

#[test]
fn invalid_utf8_too_small_2()         { expect( &[0b1100_0000, b'1']                     ).to_error(InvalidUtf8,0) }
#[test]
fn invalid_utf8_too_small_eof_2()     { expect( &[0b1100_0000]                           ).to_error(InvalidUtf8,0) }
#[test]
fn invalid_utf8_too_small_3()         { expect( &[0b1110_0000, 0b1000_0000, b'1']        ).to_error(InvalidUtf8,0..=1) }
#[test]
fn invalid_utf8_too_small_eof_3()     { expect( &[0b1110_0000, 0b1000_0000]              ).to_error(InvalidUtf8,0..=1) }
#[test]
fn invalid_utf8_too_small_4()         { expect( &[0b1110_0000, 0b1000_0000, b'1']        ).to_error(InvalidUtf8,0..=1) }
#[test]
fn invalid_utf8_too_small_eof_4()     { expect( &[0b1111_0000, 0b1000_0000, 0b1000_0000] ).to_error(InvalidUtf8,0..=2) }

#[test]
fn unsupported_and_invalid()          { expect( &[b'`', 0b1000_0000]                     ).to_error(UnsupportedCharacters,0) }
#[test]
fn unsupported_and_invalid_multiple() { expect( &[b'`', b'`', 0b1000_0000, 0b1000_0000]  ).to_error(UnsupportedCharacters,0..=1) }
#[test]
fn invalid_and_unsupported()          { expect( &[0b1000_0000, b'`']                     ).to_error(InvalidUtf8,0) }
#[test]
fn invalid_and_unsupported_multiple() { expect( &[0b1000_0000, 0b1000_0000, b'`', b'`']  ).to_error(InvalidUtf8,0..=1) }
