pub mod compiler_test;
use crate::compiler_test::*;

#[test]
fn and_true_true() {
    expect("true&&true").to_yield(true)
}
#[test]
fn and_true_false() {
    expect("true&&false").to_yield(false)
}
#[test]
fn and_false_true() {
    expect("false&&true").to_yield(false)
}
#[test]
fn and_false_false() {
    expect("false&&false").to_yield(false)
}

#[test]
fn or_true_true() {
    expect("true||true").to_yield(true)
}
#[test]
fn or_true_false() {
    expect("true||false").to_yield(true)
}
#[test]
fn or_false_true() {
    expect("false||true").to_yield(true)
}
#[test]
fn or_false_false() {
    expect("false||false").to_yield(false)
}

#[test]
fn not_true() {
    expect("!true").to_yield(false)
}
#[test]
fn not_false() {
    expect("!false").to_yield(true)
}

#[test]
fn and_equal_equal() {
    expect("1==1&&2==2").to_yield(true)
}
#[test]
fn and_ne_ne() {
    expect("1!=2&&3!=4").to_yield(true)
}
#[test]
fn and_greater_greater() {
    expect("4>3&&2>1").to_yield(true)
}
#[test]
fn and_less_less() {
    expect("1<2&&3<4").to_yield(true)
}
#[test]
fn and_ge_ge() {
    expect("4>=3&&2>=1").to_yield(true)
}
#[test]
fn and_le_le() {
    expect("1<=2&&3<=5").to_yield(true)
}

#[test]
fn or_equal_equal() {
    expect("1==2||2==2").to_yield(true)
}
#[test]
fn or_ne_ne() {
    expect("1!=1||3!=4").to_yield(true)
}
#[test]
fn or_greater_greater() {
    expect("4>5||2>1").to_yield(true)
}
#[test]
fn or_less_less() {
    expect("3<2||3<4").to_yield(true)
}
#[test]
fn or_ge_ge() {
    expect("4>=5||2>=1").to_yield(true)
}
#[test]
fn or_le_le() {
    expect("4<=5||2<=1").to_yield(true)
}

#[test]
fn and_or_ge_add_mul_true() {
    expect("false||true&&7<=1+2*3").to_yield(true)
}
#[test]
fn and_or_ge_add_mul_false() {
    expect("false||true&&8<=1+2*3").to_yield(false)
}
