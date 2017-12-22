#[macro_use]
pub mod compiler_test;

compiler_tests! {
    empty: "" => value(nothing),
    space: " " => value(nothing),
    newline: "\n" => value(nothing),
    newline_newline: "\n" => value(nothing),
    newline_cr: "\n\r" => value(nothing),
    newline_crlf: "\n\r\n" => value(nothing),
    cr: "\r" => value(nothing),
    cr_cr: "\r\r" => value(nothing),
    cr_crlf: "\r\r\n" => value(nothing),
    crlf: "\r\n" => value(nothing),
    crlf_newline: "\r\n\n" => value(nothing),
    crlf_cr: "\r\n\r" => value(nothing),
    crlf_crlf: "\r\n\r\n" => value(nothing),
}
