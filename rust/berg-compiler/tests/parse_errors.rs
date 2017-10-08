#[macro_use]
mod compiler_test;

compiler_tests! {
    unsupported_characters: "`" => error(UnsupportedCharacters@0),

    invalid_utf8_no_leading_byte: (&vec![0b1000_0000])                           => error(InvalidUtf8@0),
    invalid_utf8_invalid_byte:    (&vec![0b1111_1000])                           => error(InvalidUtf8@0),
    invalid_utf8_multiple:        (&vec![0b1000_0000, 0b1111_1000, 0b1000_0000]) => error(InvalidUtf8@[0-2]),

    invalid_utf8_too_small_2:     (&vec![0b1100_0000, b'1'])                     => error(InvalidUtf8@0),
    invalid_utf8_too_small_eof_2: (&vec![0b1100_0000])                           => error(InvalidUtf8@0),
    invalid_utf8_too_small_3:     (&vec![0b1110_0000, 0b1000_0000, b'1'])        => error(InvalidUtf8@[0-1]),
    invalid_utf8_too_small_eof_3: (&vec![0b1110_0000, 0b1000_0000])              => error(InvalidUtf8@[0-1]),
    invalid_utf8_too_small_4:     (&vec![0b1110_0000, 0b1000_0000, b'1'])        => error(InvalidUtf8@[0-1]),
    invalid_utf8_too_small_eof_4: (&vec![0b1111_0000, 0b1000_0000, 0b1000_0000]) => error(InvalidUtf8@[0-2]),

    unsupported_and_invalid: (&vec![b'`', 0b1000_0000]) => errors(UnsupportedCharacters@0, InvalidUtf8@1),
    invalid_and_unsupported: (&vec![0b1000_0000, b'`']) => errors(InvalidUtf8@0, UnsupportedCharacters@1),
}
