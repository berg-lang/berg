#[macro_use]
pub mod compiler_test;

compiler_tests! {
    empty: "" => type(nothing),
    space: " " => type(nothing),
    newline: "\n" => type(nothing),
    newline_newline: "\n" => type(nothing),
    newline_cr: "\n\r" => type(nothing),
    newline_crlf: "\n\r\n" => type(nothing),
    cr: "\r" => type(nothing),
    cr_cr: "\r\r" => type(nothing),
    cr_crlf: "\r\r\n" => type(nothing),
    crlf: "\r\n" => type(nothing),
    crlf_newline: "\r\n\n" => type(nothing),
    crlf_cr: "\r\n\r" => type(nothing),
    crlf_crlf: "\r\n\r\n" => type(nothing),
}
