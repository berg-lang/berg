pub mod compiler_test;
use crate::compiler_test::*;

#[test]
fn left_newline() {
    expect("\n1").to_yield(1)
}
#[test]
fn right_newline() {
    expect("1\n").to_yield(1)
}
#[test]
fn both_newline() {
    expect("\n1\n").to_yield(1)
}

#[test]
fn left_double_newline() {
    expect("\n\n1").to_yield(1)
}
#[test]
fn right_double_newline() {
    expect("1\n\n").to_yield(1)
}
#[test]
fn both_double_newline() {
    expect("\n\n1\n\n").to_yield(1)
}
#[test]
fn double_newline_between() {
    expect("1\n\n2").to_yield(2)
}

#[test]
fn paren_newline_all_over() {
    expect("\n(\n(\n)\n)\n").to_yield(Nothing)
}

#[test]
fn newline_sequence() {
    expect("1\n2").to_yield(2)
}
#[test]
fn newline_sequence_add() {
    expect("1+1\n2+2").to_yield(4)
}
#[test]
fn newline_sequence_or_and_ge_plus_mul() {
    expect("1*2+3>=4&&true||false\nfalse||true&&4>=3+2*1").to_yield(false)
}
#[test]
fn newline_sequence_or_and_le_plus_mul() {
    expect("1*2+3<=4&&true||false\nfalse||true&&4<=3+2*1").to_yield(true)
}
