use crate::*;

#[test]
fn comment_char_only() {
    expect("#").to_yield(tuple!())
}
#[test]
fn comment() {
    expect("# comment").to_yield(tuple!())
}
#[test]
fn only_one_char() {
    expect("#c").to_yield(tuple!())
}
#[test]
fn one_space_only() {
    expect("# ").to_yield(tuple!())
}
#[test]
fn space_only() {
    expect("#   ").to_yield(tuple!())
}
#[test]
fn newline_only() {
    expect("#\n").to_yield(tuple!())
}
#[test]
fn cr_only() {
    expect("#\r").to_yield(tuple!())
}
#[test]
fn crlf_only() {
    expect("#\r\n").to_yield(tuple!())
}
#[test]
fn multiple() {
    expect("#one\n#two").to_yield(tuple!())
}
#[test]
fn expression_before_same_line() {
    expect("1 # one").to_yield(1)
}
#[test]
fn expression_before() {
    expect(
        "
        1
        # comment
    ",
    )
    .to_yield(1)
}
#[test]
fn expression_after() {
    expect(
        "
        # comment
        1
    ",
    )
    .to_yield(1)
}
#[test]
fn expression_immediately_after() {
    expect("# comment\n1+1").to_yield(2)
}
#[test]
fn comment_between_sequence() {
    expect(
        "
        :x = 1
        # comment
        x
    ",
    )
    .to_yield(1)
}
#[test]
fn comment_ends_term() {
    expect("2 * 1+1#comment").to_yield(4)
}
#[test]
fn term_starts_after_comment() {
    expect("#comment\n1+1 * 2").to_yield(4)
}
#[test]
fn comment_before_infix() {
    expect(
        "
        1
        # comment
        + 2
    ",
    )
    .to_yield(3)
}
#[test]
fn comment_after_infix() {
    expect(
        "
        1 +
        # comment
        2
    ",
    )
    .to_yield(3)
}
#[test]
fn comment_after_infix_same_line() {
    expect(
        "
        1 + # comment
        2
    ",
    )
    .to_yield(3)
}
#[test]
fn comment_immediately_after_infix_same_line() {
    expect("1 +#comment\n2").to_yield(3)
}
#[test]
fn comment_with_infix_operator_same_line() {
    expect(
        "
        1
        + # comment
        2
    ",
    )
    .to_yield(3)
}
#[test]
fn comment_immediately_with_infix_operator_same_line() {
    expect(
        "
        1
        +# comment
        2
    ",
    )
    .to_yield(3)
}
#[test]
fn comment_in_curly_braces() {
    expect(
        "
        {
            :x = 1
            # comment
            x
        }
    ",
    )
    .to_yield(1)
}
#[test]
fn comment_in_parentheses() {
    expect(
        "
        {
            :x = 1
            # comment
            x
        }
    ",
    )
    .to_yield(1)
}
#[test]
fn comment_does_not_end_curly_braces() {
    expect(
        "
        {
        # }
        }
    ",
    )
    .to_yield(tuple!())
}
#[test]
fn comment_on_same_line_does_not_end_curly_braces() {
    expect(
        "
        { # }
        }
    ",
    )
    .to_yield(tuple!())
}
#[test]
fn comment_does_not_end_parentheses() {
    expect(
        "
        (
        # )
        )
    ",
    )
    .to_yield(tuple!())
}
#[test]
fn comment_on_same_line_does_not_end_parentheses() {
    expect(
        "
        ( # )
        )
    ",
    )
    .to_yield(tuple!())
}

#[test]
fn unsupported() {
    expect("#`").to_yield(tuple!())
}
#[test]
fn unsupported_multibyte() {
    expect("#⌂").to_yield(tuple!())
}
#[test]
fn unsupported_multiple() {
    expect("#⌂`⌂").to_yield(tuple!())
}
#[test]
fn unsupported_then_ok() {
    expect("#`1").to_yield(tuple!())
}
#[test]
fn unsupported_multibyte_then_ok() {
    expect("#⌂1").to_yield(tuple!())
}
#[test]
fn unsupported_multiple_then_ok() {
    expect("#⌂`⌂1").to_yield(tuple!())
}
#[test]
fn ok_then_unsupported() {
    expect("#1`").to_yield(tuple!())
}
#[test]
fn ok_then_unsupported_multibyte() {
    expect("#1⌂").to_yield(tuple!())
}
#[test]
fn ok_then_unsupported_multiple() {
    expect("#1⌂`⌂").to_yield(tuple!())
}

#[test]
fn invalid_utf8_no_leading_byte() {
    expect(&[b'#', 0b1000_0000]).to_yield(tuple!())
}
#[test]
fn invalid_utf8_invalid_byte() {
    expect(&[b'#', 0b1111_1000]).to_yield(tuple!())
}
#[test]
fn invalid_utf8_multiple() {
    expect(&[b'#', 0b1000_0000, 0b1111_1000, 0b1000_0000]).to_yield(tuple!())
}
#[test]
fn invalid_utf8_multiple_then_ok() {
    expect(&[b'#', 0b1000_0000, 0b1111_1000, 0b1000_0000, b'1']).to_yield(tuple!())
}

#[test]
fn invalid_utf8_too_small_2() {
    expect(&[b'#', 0b1100_0000, b'1']).to_yield(tuple!())
}
#[test]
fn invalid_utf8_too_small_eof_2() {
    expect(&[b'#', 0b1100_0000]).to_yield(tuple!())
}
#[test]
fn invalid_utf8_too_small_3() {
    expect(&[b'#', 0b1110_0000, 0b1000_0000, b'1']).to_yield(tuple!())
}
#[test]
fn invalid_utf8_too_small_eof_3() {
    expect(&[b'#', 0b1110_0000, 0b1000_0000]).to_yield(tuple!())
}
#[test]
fn invalid_utf8_too_small_4() {
    expect(&[b'#', 0b1110_0000, 0b1000_0000, b'1']).to_yield(tuple!())
}
#[test]
fn invalid_utf8_too_small_eof_4() {
    expect(&[b'#', 0b1111_0000, 0b1000_0000, 0b1000_0000]).to_yield(tuple!())
}

#[test]
fn unsupported_then_invalid() {
    expect(&[b'#', b'`', 0b1000_0000]).to_yield(tuple!())
}
#[test]
fn unsupported_then_invalid_multibyte() {
    // 0xE28C82 is ⌂
    expect(&[b'#', 0xE2, 0x8C, 0x82, 0b1000_0000]).to_yield(tuple!())
}
#[test]
fn unsupported_then_invalid_multiple() {
    // 0xE28C82 is ⌂
    expect(&[
        b'#',
        0xE2,
        0x8C,
        0x82,
        b'`',
        0xE2,
        0x8C,
        0x82,
        0b1000_0000,
        0b1000_0000,
    ])
    .to_yield(tuple!())
}
#[test]
fn invalid_then_unsupported() {
    expect(&[b'#', 0b1000_0000, b'`']).to_yield(tuple!())
}
#[test]
fn invalid_then_unsupported_multibyte() {
    // 0xE28C82 is ⌂
    expect(&[b'#', 0b1000_0000, 0xE2, 0x8C, 0x82]).to_yield(tuple!())
}
#[test]
fn invalid_then_unsupported_multiple() {
    // 0xE28C82 is ⌂
    expect(&[
        b'#',
        0b1000_0000,
        0b1000_0000,
        0xE2,
        0x8C,
        0x82,
        b'`',
        0xE2,
        0x8C,
        0x82,
    ])
    .to_yield(tuple!())
}
