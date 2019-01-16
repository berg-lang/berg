pub mod compiler_test;
use crate::compiler_test::*;

// number/boolean
#[test]
fn greater_than_0_false() {
    expect("0>false").to_error(BadType, 2..=6)
}
#[test]
fn greater_than_false_0() {
    expect("false>0").to_error(UnsupportedOperator, 5)
}
#[test]
fn greater_than_1_false() {
    expect("1>false").to_error(BadType, 2..=6)
}
#[test]
fn greater_than_false_1() {
    expect("false>1").to_error(UnsupportedOperator, 5)
}
#[test]
fn greater_than_0_true() {
    expect("0>true").to_error(BadType, 2..=5)
}
#[test]
fn greater_than_true_0() {
    expect("true>0").to_error(UnsupportedOperator, 4)
}
#[test]
fn greater_than_1_true() {
    expect("1>true").to_error(BadType, 2..=5)
}
#[test]
fn greater_than_true_1() {
    expect("true>1").to_error(UnsupportedOperator, 4)
}

#[test]
fn greater_or_equal_0_false() {
    expect("0>=false").to_error(BadType, 3..=7)
}
#[test]
fn greater_or_equal_false_0() {
    expect("false>=0").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn greater_or_equal_1_false() {
    expect("1>=false").to_error(BadType, 3..=7)
}
#[test]
fn greater_or_equal_false_1() {
    expect("false>=1").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn greater_or_equal_0_true() {
    expect("0>=true").to_error(BadType, 3..=6)
}
#[test]
fn greater_or_equal_true_0() {
    expect("true>=0").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn greater_or_equal_1_true() {
    expect("1>=true").to_error(BadType, 3..=6)
}
#[test]
fn greater_or_equal_true_1() {
    expect("true>=1").to_error(UnsupportedOperator, 4..=5)
}

#[test]
fn less_than_0_false() {
    expect("0<false").to_error(BadType, 2..=6)
}
#[test]
fn less_than_false_0() {
    expect("false<0").to_error(UnsupportedOperator, 5)
}
#[test]
fn less_than_1_false() {
    expect("1<false").to_error(BadType, 2..=6)
}
#[test]
fn less_than_false_1() {
    expect("false<1").to_error(UnsupportedOperator, 5)
}
#[test]
fn less_than_0_true() {
    expect("0<true").to_error(BadType, 2..=5)
}
#[test]
fn less_than_true_0() {
    expect("true<0").to_error(UnsupportedOperator, 4)
}
#[test]
fn less_than_1_true() {
    expect("1<true").to_error(BadType, 2..=5)
}
#[test]
fn less_than_true_1() {
    expect("true<1").to_error(UnsupportedOperator, 4)
}

#[test]
fn less_or_equal_0_false() {
    expect("0<=false").to_error(BadType, 3..=7)
}
#[test]
fn less_or_equal_false_0() {
    expect("false<=0").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn less_or_equal_1_false() {
    expect("1<=false").to_error(BadType, 3..=7)
}
#[test]
fn less_or_equal_false_1() {
    expect("false<=1").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn less_or_equal_0_true() {
    expect("0<=true").to_error(BadType, 3..=6)
}
#[test]
fn less_or_equal_true_0() {
    expect("true<=0").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn less_or_equal_1_true() {
    expect("1<=true").to_error(BadType, 3..=6)
}
#[test]
fn less_or_equal_true_1() {
    expect("true<=1").to_error(UnsupportedOperator, 4..=5)
}

// number/nothing
#[test]
fn greater_than_0_nothing() {
    expect("0>()").to_error(BadType, 2..=3)
}
#[test]
fn greater_than_nothing_0() {
    expect("()>0").to_error(UnsupportedOperator, 2)
}
#[test]
fn greater_or_equal_0_nothing() {
    expect("0>=()").to_error(BadType, 3..=4)
}
#[test]
fn greater_or_equal_nothing_0() {
    expect("()>=0").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn less_than_0_nothing() {
    expect("0<()").to_error(BadType, 2..=3)
}
#[test]
fn less_than_nothing_0() {
    expect("()<0").to_error(UnsupportedOperator, 2)
}
#[test]
fn less_or_equal_0_nothing() {
    expect("0<=()").to_error(BadType, 3..=4)
}
#[test]
fn less_or_equal_nothing_0() {
    expect("()<=0").to_error(UnsupportedOperator, 2..=3)
}

// number/error
#[test]
fn greater_than_0_error() {
    expect("0>1/0").to_error(DivideByZero, 4)
}
#[test]
fn greater_than_error_0() {
    expect("1/0>0").to_error(DivideByZero, 2)
}
#[test]
fn greater_or_equal_0_error() {
    expect("0>=1/0").to_error(DivideByZero, 5)
}
#[test]
fn greater_or_equal_error_0() {
    expect("1/0>=0").to_error(DivideByZero, 2)
}
#[test]
fn less_than_0_error() {
    expect("0<1/0").to_error(DivideByZero, 4)
}
#[test]
fn less_than_error_0() {
    expect("1/0<0").to_error(DivideByZero, 2)
}
#[test]
fn less_or_equal_0_error() {
    expect("0<=1/0").to_error(DivideByZero, 5)
}
#[test]
fn less_or_equal_error_0() {
    expect("1/0<=0").to_error(DivideByZero, 2)
}

// booleans
#[test]
fn greater_than_true_true() {
    expect("true>true").to_error(UnsupportedOperator, 4)
}
#[test]
fn greater_than_true_false() {
    expect("true>false").to_error(UnsupportedOperator, 4)
}
#[test]
fn greater_than_false_true() {
    expect("false>true").to_error(UnsupportedOperator, 5)
}
#[test]
fn greater_than_false_false() {
    expect("false>false").to_error(UnsupportedOperator, 5)
}

#[test]
fn greater_or_equal_true_true() {
    expect("true>=true").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn greater_or_equal_true_false() {
    expect("true>=false").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn greater_or_equal_false_true() {
    expect("false>=true").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn greater_or_equal_false_false() {
    expect("false>=false").to_error(UnsupportedOperator, 5..=6)
}

#[test]
fn less_than_true_true() {
    expect("true<true").to_error(UnsupportedOperator, 4)
}
#[test]
fn less_than_true_false() {
    expect("true<false").to_error(UnsupportedOperator, 4)
}
#[test]
fn less_than_false_true() {
    expect("false<true").to_error(UnsupportedOperator, 5)
}
#[test]
fn less_than_false_false() {
    expect("false<false").to_error(UnsupportedOperator, 5)
}

#[test]
fn less_or_equal_true_true() {
    expect("true<=true").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn less_or_equal_true_false() {
    expect("true<=false").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn less_or_equal_false_true() {
    expect("false<=true").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn less_or_equal_false_false() {
    expect("false<=false").to_error(UnsupportedOperator, 5..=6)
}

// boolean/nothing
#[test]
fn greater_than_true_nothing() {
    expect("true>()").to_error(UnsupportedOperator, 4)
}
#[test]
fn greater_than_false_nothing() {
    expect("false>()").to_error(UnsupportedOperator, 5)
}
#[test]
fn greater_than_nothing_true() {
    expect("()>true").to_error(UnsupportedOperator, 2)
}
#[test]
fn greater_than_nothing_false() {
    expect("()>false").to_error(UnsupportedOperator, 2)
}
#[test]
fn greater_or_equal_true_nothing() {
    expect("true>=()").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn greater_or_equal_false_nothing() {
    expect("false>=()").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn greater_or_equal_nothing_true() {
    expect("()>=true").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn greater_or_equal_nothing_false() {
    expect("()>=false").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn less_than_true_nothing() {
    expect("true<()").to_error(UnsupportedOperator, 4)
}
#[test]
fn less_than_false_nothing() {
    expect("false<()").to_error(UnsupportedOperator, 5)
}
#[test]
fn less_than_nothing_true() {
    expect("()<true").to_error(UnsupportedOperator, 2)
}
#[test]
fn less_than_nothing_false() {
    expect("()<false").to_error(UnsupportedOperator, 2)
}
#[test]
fn less_or_equal_true_nothing() {
    expect("true<=()").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn less_or_equal_false_nothing() {
    expect("false<=()").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn less_or_equal_nothing_true() {
    expect("()<=true").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn less_or_equal_nothing_false() {
    expect("()<=false").to_error(UnsupportedOperator, 2..=3)
}

// boolean/error
#[test]
fn greater_than_true_error() {
    expect("true>1/0").to_error(UnsupportedOperator, 4)
}
#[test]
fn greater_than_false_error() {
    expect("false>1/0").to_error(UnsupportedOperator, 5)
}
#[test]
fn greater_than_error_true() {
    expect("1/0>true").to_error(DivideByZero, 2)
}
#[test]
fn greater_than_error_false() {
    expect("1/0>false").to_error(DivideByZero, 2)
}
#[test]
fn greater_or_equal_true_error() {
    expect("true>=1/0").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn greater_or_equal_false_error() {
    expect("false>=1/0").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn greater_or_equal_error_true() {
    expect("1/0>=true").to_error(DivideByZero, 2)
}
#[test]
fn greater_or_equal_error_false() {
    expect("1/0>=false").to_error(DivideByZero, 2)
}
#[test]
fn less_than_true_error() {
    expect("true<1/0").to_error(UnsupportedOperator, 4)
}
#[test]
fn less_than_false_error() {
    expect("false<1/0").to_error(UnsupportedOperator, 5)
}
#[test]
fn less_than_error_true() {
    expect("1/0<true").to_error(DivideByZero, 2)
}
#[test]
fn less_than_error_false() {
    expect("1/0<false").to_error(DivideByZero, 2)
}
#[test]
fn less_or_equal_true_error() {
    expect("true<=1/0").to_error(UnsupportedOperator, 4..=5)
}
#[test]
fn less_or_equal_false_error() {
    expect("false<=1/0").to_error(UnsupportedOperator, 5..=6)
}
#[test]
fn less_or_equal_error_true() {
    expect("1/0<=true").to_error(DivideByZero, 2)
}
#[test]
fn less_or_equal_error_false() {
    expect("1/0<=false").to_error(DivideByZero, 2)
}

// nothing
#[test]
fn greater_than_nothing_nothing() {
    expect("()>()").to_error(UnsupportedOperator, 2)
}
#[test]
fn greater_or_equal_nothing_nothing() {
    expect("()>=()").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn less_than_nothing_nothing() {
    expect("()<()").to_error(UnsupportedOperator, 2)
}
#[test]
fn less_or_equal_nothing_nothing() {
    expect("()<=()").to_error(UnsupportedOperator, 2..=3)
}

// nothing/error
#[test]
fn greater_than_nothing_error() {
    expect("()>1/0").to_error(UnsupportedOperator, 2)
}
#[test]
fn greater_than_error_nothing() {
    expect("1/0>()").to_error(DivideByZero, 2)
}
#[test]
fn greater_or_equal_nothing_error() {
    expect("()>=1/0").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn greater_or_equal_error_nothing() {
    expect("1/0>=()").to_error(DivideByZero, 2)
}
#[test]
fn less_than_nothing_error() {
    expect("()<1/0").to_error(UnsupportedOperator, 2)
}
#[test]
fn less_than_error_nothing() {
    expect("1/0<()").to_error(DivideByZero, 2)
}
#[test]
fn less_or_equal_error_nothing() {
    expect("()<=1/0").to_error(UnsupportedOperator, 2..=3)
}
#[test]
fn less_or_equal_nothing_error() {
    expect("1/0<=()").to_error(DivideByZero, 2)
}

// errors
#[test]
fn greater_than_error_error() {
    expect("1/0>1/0").to_error(DivideByZero, 2)
}
#[test]
fn greater_or_equal_error_error() {
    expect("1/0>=1/0").to_error(DivideByZero, 2)
}
#[test]
fn less_than_error_error() {
    expect("1/0<1/0").to_error(DivideByZero, 2)
}
#[test]
fn less_or_equal_error_error() {
    expect("1/0<=1/0").to_error(DivideByZero, 2)
}
