pub mod compiler_test;
use crate::compiler_test::*;

#[test]
fn empty() {
    expect("").to_yield(tuple!())
}
#[test]
fn space() {
    expect(" ").to_yield(tuple!())
}
#[test]
fn newline() {
    expect("\n").to_yield(tuple!())
}
#[test]
fn newline_newline() {
    expect("\n").to_yield(tuple!())
}
#[test]
fn newline_cr() {
    expect("\n\r").to_yield(tuple!())
}
#[test]
fn newline_crlf() {
    expect("\n\r\n").to_yield(tuple!())
}
#[test]
fn cr() {
    expect("\r").to_yield(tuple!())
}
#[test]
fn cr_cr() {
    expect("\r\r").to_yield(tuple!())
}
#[test]
fn cr_crlf() {
    expect("\r\r\n").to_yield(tuple!())
}
#[test]
fn crlf() {
    expect("\r\n").to_yield(tuple!())
}
#[test]
fn crlf_newline() {
    expect("\r\n\n").to_yield(tuple!())
}
#[test]
fn crlf_cr() {
    expect("\r\n\r").to_yield(tuple!())
}
#[test]
fn crlf_crlf() {
    expect("\r\n\r\n").to_yield(tuple!())
}
