#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    empty: "" => value(Nothing),
    space: " " => value(Nothing),
    newline: "\n" => value(Nothing),
    newline_newline: "\n" => value(Nothing),
    newline_cr: "\n\r" => value(Nothing),
    newline_crlf: "\n\r\n" => value(Nothing),
    cr: "\r" => value(Nothing),
    cr_cr: "\r\r" => value(Nothing),
    cr_crlf: "\r\r\n" => value(Nothing),
    crlf: "\r\n" => value(Nothing),
    crlf_newline: "\r\n\n" => value(Nothing),
    crlf_cr: "\r\n\r" => value(Nothing),
    crlf_crlf: "\r\n\r\n" => value(Nothing),
}
