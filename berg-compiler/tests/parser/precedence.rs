use crate::*;

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
#[test]
fn addmul_missing_operator_precedence() {
    expect("1 * + 3").to_error(MissingOperand, 2)
}
#[test]
fn muladd_missing_operator_precedence() {
    expect("1 + * 3").to_error(MissingOperand, 4)
}
#[test]
fn addparen_missing_operator_precedence() {
    expect("(1 + )").to_error(MissingOperand, 3)
}
#[test]
fn parenadd_missing_operator_precedence() {
    expect("( + 1)").to_error(MissingOperand, 2)
}


#[test]
fn trailing_neg() {
    expect("0-").to_error(UnsupportedOperator, 1)
}
#[test]
fn trailing_pos() {
    expect("0+").to_error(UnsupportedOperator, 1)
}
#[test]
fn sub_only() {
    expect("-").to_error(MissingOperand, 0)
}
#[test]
fn add_only() {
    expect("+").to_error(MissingOperand, 0)
}
#[test]
fn plus_minus() {
    expect("1+-2").to_error(UnsupportedOperator, 1..=2)
}
#[test]
fn comma_sequence_add() {
    expect("1+1+1,2+2+2").to_yield(tuple!(3, 6))
}
#[test]
fn comma_sequence_or_and_ge_plus_mul() {
    expect("1*2+3>=4&&true||false,false||true&&4>=3+2*1").to_yield(tuple!(true, false))
}
#[test]
fn comma_sequence_or_and_le_plus_mul() {
    expect("1*2+3<=4&&true||false,false||true&&4<=3+2*1").to_yield(tuple!(false, true))
}
