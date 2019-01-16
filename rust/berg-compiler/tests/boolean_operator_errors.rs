pub mod compiler_test;
use crate::compiler_test::*;

#[test]
fn not_equal() {
    expect("!1==1").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_ne() {
    expect("!1!=1").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_greater() {
    expect("!1>1").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_less() {
    expect("!1<1").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_ge() {
    expect("!1>=1").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_le() {
    expect("!1<=1").to_error(UnsupportedOperator, 0)
}

// && errors
#[test]
fn and_true_1() {
    expect("true&&1").to_error(BadType, 6)
}
#[test]
fn and_false_1() {
    expect("false&&1").to_yield(false)
}
#[test]
fn and_1_true() {
    expect("1&&true").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn and_1_false() {
    expect("1&&false").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn and_true_error() {
    expect("true&&(1/0)").to_error(DivideByZero, 9)
}
#[test]
fn and_false_error() {
    expect("false&&(1/0)").to_yield(false)
}
#[test]
fn and_error_true() {
    expect("(1/0)&&true").to_error(DivideByZero, 3)
}
#[test]
fn and_error_false() {
    expect("(1/0)&&false").to_error(DivideByZero, 3)
}
#[test]
fn and_true_nothing() {
    expect("true&&()").to_error(BadType, 6..=7)
}
#[test]
fn and_false_nothing() {
    expect("false&&()").to_yield(false)
}
#[test]
fn and_nothing_true() {
    expect("()&&true").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn and_nothing_false() {
    expect("()&&false").to_error(UnsupportedOperator, 2..=3)
}

#[test]
fn and_1_1() {
    expect("1&&1").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn and_1_nothing() {
    expect("1&&()").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn and_nothing_1() {
    expect("()&&1").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn and_1_error() {
    expect("1&&(1/0)").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn and_error_1() {
    expect("(1/0)&&1").to_error(DivideByZero, 3)
}
#[test]
fn and_nothing_nothing() {
    expect("()&&()").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn and_nothing_error() {
    expect("()&&(1/0)").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn and_error_nothing() {
    expect("(1/0)&&()").to_error(DivideByZero, 3)
}
#[test]
fn and_error_error() {
    expect("(1/0)&&(1/0)").to_error(DivideByZero, 3)
}

// || errors
#[test]
fn or_true_1() {
    expect("true||1").to_yield(true)
}
#[test]
fn or_false_1() {
    expect("false||1").to_error(BadType, 7)
}
#[test]
fn or_1_true() {
    expect("1||true").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn or_1_false() {
    expect("1||false").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn or_true_error() {
    expect("true||(1/0)").to_yield(true)
}
#[test]
fn or_false_error() {
    expect("false||(1/0)").to_error(DivideByZero, 10)
}
#[test]
fn or_error_true() {
    expect("(1/0)||true").to_error(DivideByZero, 3)
}
#[test]
fn or_error_false() {
    expect("(1/0)||false").to_error(DivideByZero, 3)
}
#[test]
fn or_true_nothing() {
    expect("true||()").to_yield(true)
}
#[test]
fn or_false_nothing() {
    expect("false||()").to_error(BadType, 7..=8)
}
#[test]
fn or_nothing_true() {
    expect("()||true").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn or_nothing_false() {
    expect("()||false").to_error(UnsupportedOperator, 2..=3)
}

#[test]
fn or_1_1() {
    expect("1||1").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn or_1_nothing() {
    expect("1||()").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn or_nothing_1() {
    expect("()||1").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn or_1_error() {
    expect("1||(1/0)").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn or_error_1() {
    expect("(1/0)||1").to_error(DivideByZero, 3)
}
#[test]
fn or_nothing_nothing() {
    expect("()||()").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn or_nothing_error() {
    expect("()||(1/0)").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn or_error_nothing() {
    expect("(1/0)||()").to_error(DivideByZero, 3)
}
#[test]
fn or_error_error() {
    expect("(1/0)||(1/0)").to_error(DivideByZero, 3)
}

// ! errors
#[test]
fn not_1() {
    expect("!1").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_nothing() {
    expect("!()").to_error(UnsupportedOperator, 0)
}
#[test]
fn not_error() {
    expect("!(1/0)").to_error(DivideByZero, 4)
}
