use crate::*;

//
// Test declarations with references
//

#[test]
fn assign_ref() {
    expect(":a = 1; a").to_yield(1)
}
#[test]
fn reassign_ref() {
    expect(":a = 1; a = 2; a").to_yield(2)
}
#[test]
fn two_fields_ref() {
    expect(":a = 1; :b = 2; a + b").to_yield(3)
}

#[test]
fn assign_plus_ref() {
    expect(":a = 1;  a += 2; a").to_yield(3)
}
#[test]
fn assign_minus_ref() {
    expect(":a = 3;  a -= 2; a").to_yield(1)
}
#[test]
fn assign_times_ref() {
    expect(":a = 2;  a *= 3; a").to_yield(6)
}
#[test]
fn assign_divide_ref() {
    expect(":a = 12; a /= 4; a").to_yield(3)
}
#[test]
fn increment_post_ref() {
    expect(":a = 1; a++; a").to_yield(2)
}
#[test]
fn increment_pre_ref() {
    expect(":a = 1; ++a; a").to_yield(2)
}
#[test]
fn decrement_post_ref() {
    expect(":a = 1; a--; a").to_yield(0)
}
#[test]
fn decrement_pre_ref() {
    expect(":a = 1; --a; a").to_yield(0)
}

#[test]
fn assign_and_true_true_ref() {
    expect(":a = true;  a &&= true;  a").to_yield(true)
}
#[test]
fn assign_and_true_false_ref() {
    expect(":a = true;  a &&= false; a").to_yield(false)
}
#[test]
fn assign_and_false_true_ref() {
    expect(":a = false; a &&= true;  a").to_yield(false)
}
#[test]
fn assign_and_false_false_ref() {
    expect(":a = false; a &&= false; a").to_yield(false)
}
#[test]
fn assign_or_true_true_ref() {
    expect(":a = true;  a ||= true;  a").to_yield(true)
}
#[test]
fn assign_or_true_false_ref() {
    expect(":a = true;  a ||= false; a").to_yield(true)
}
#[test]
fn assign_or_false_true_ref() {
    expect(":a = false; a ||= true;  a").to_yield(true)
}
#[test]
fn assign_or_false_false_ref() {
    expect(":a = false; a ||= false; a").to_yield(false)
}

//
// Test precedence
//
#[test]
fn assign_precedence() {
    expect("a = false; b = false || true && 14 == 2 + 3 * 4; b").to_yield(true)
}
#[test]
fn assign_plus_precedence() {
    expect(":b = 1; b += 2 + 3 * 4; b").to_yield(15)
}
#[test]
fn assign_minus_precedence() {
    expect(":b = 1; b -= 2 + 3 * 4; b").to_yield(-13)
}
#[test]
fn assign_multiply_precedence() {
    expect(":b = 2; b *= 2 + 3 * 4; b").to_yield(28)
}
#[test]
fn assign_divide_precedence() {
    expect(":b = 28; b /= 2 + 3 * 4; b").to_yield(2)
}
#[test]
fn assign_and_precedence() {
    expect("a = false; b = true; b &&= false || true && 14 == 2 + 3 * 4; b").to_yield(true)
}
#[test]
fn assign_or_precedence() {
    expect("a = false; b = false; b ||= false || true && 14 == 2 + 3 * 4; b").to_yield(true)
}

//
// Test declarations without references
//

#[test]
fn bare_declaration() {
    expect(":a").to_error(FieldNotSet, 1)
}
#[test]
fn assign() {
    expect(":a = 1").to_yield(tuple!())
}
#[test]
fn reassign() {
    expect(":a = 1;  a = 2").to_yield(tuple!())
}

#[test]
fn assign_plus() {
    expect(":a = 1;  a += 2").to_yield(tuple!())
}
#[test]
fn assign_minus() {
    expect(":a = 3;  a -= 2").to_yield(tuple!())
}
#[test]
fn assign_times() {
    expect(":a = 2;  a *= 3").to_yield(tuple!())
}
#[test]
fn assign_divide() {
    expect(":a = 12; a /= 4").to_yield(tuple!())
}
#[test]
fn assign_and() {
    expect(":a = true;  a &&= false").to_yield(tuple!())
}
#[test]
fn assign_or() {
    expect(":a = false; a ||= true").to_yield(tuple!())
}
#[test]
fn increment_post() {
    expect(":a = 1;  a++").to_yield(tuple!())
}
#[test]
fn increment_pre() {
    expect(":a = 1;  ++a").to_yield(tuple!())
}
#[test]
fn decrement_post() {
    expect(":a = 1;  a--").to_yield(tuple!())
}
#[test]
fn decrement_pre() {
    expect(":a = 1;  --a").to_yield(tuple!())
}

//
// Test behavior of references to other fields of the same name
//

#[test]
fn assign_prev_ref() {
    expect(":a = 1; :a = a + 1; a").to_yield(2)
}
#[test]
fn reassign_prev_ref() {
    expect(":a = 1; a = a + 1; a").to_yield(2)
}

#[test]
fn assign_plus_prev_ref() {
    expect(":a = 3;  a += a; a").to_yield(6)
}
#[test]
fn assign_minus_prev_ref() {
    expect(":a = 3;  a -= a; a").to_yield(0)
}
#[test]
fn assign_times_prev_ref() {
    expect(":a = 3;  a *= a; a").to_yield(9)
}
#[test]
fn assign_divide_prev_ref() {
    expect(":a = 3; a /= a; a").to_yield(1)
}
#[test]
fn assign_and_prev_ref_true() {
    expect(":a = true;  a &&= a; a").to_yield(true)
}
#[test]
fn assign_and_prev_ref_false() {
    expect(":a = false; a &&= a; a").to_yield(false)
}
#[test]
fn assign_or_prev_ref_true() {
    expect(":a = true; a ||= a; a").to_yield(true)
}
#[test]
fn assign_or_prev_ref_false() {
    expect(":a = false; a ||= a; a").to_yield(false)
}

//
// Test missing syntax
//
#[test]
fn assign_missing_right() {
    expect(":a = ; a").to_error(FieldNotSet, 1)
}
#[test]
fn reassign_missing_right() {
    expect("a = ; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_plus_missing_right() {
    expect(":a += ; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_missing_right() {
    expect(":a -= ; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_multiply_missing_right() {
    expect(":a *= ; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_missing_right() {
    expect(":a /= ; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_missing_right() {
    expect(":a &&= ; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_missing_right() {
    expect(":a ||= ; a").to_error(FieldNotSet, 1)
}

#[test]
fn assign_missing_left() {
    expect("= 1").to_error(MissingOperand, 0)
}
#[test]
fn assign_plus_missing_left() {
    expect("+= 1").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_minus_missing_left() {
    expect("-= 1").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_multiply_missing_left() {
    expect("*= 1").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_divide_missing_left() {
    expect("/= 1").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_and_missing_left() {
    expect("&&= false").to_error(MissingOperand, 0..=2)
}
#[test]
fn assign_or_missing_left() {
    expect("||= true").to_error(MissingOperand, 0..=2)
}

#[test]
fn assign_missing_both() {
    expect("=").to_error(MissingOperand, 0)
}
#[test]
fn assign_plus_missing_both() {
    expect("+=").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_minus_missing_both() {
    expect("-=").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_multiply_missing_both() {
    expect("*=").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_divide_missing_both() {
    expect("/=").to_error(MissingOperand, 0..=1)
}
#[test]
fn assign_and_missing_both() {
    expect("&&=").to_error(MissingOperand, 0..=2)
}
#[test]
fn assign_or_missing_both() {
    expect("||=").to_error(MissingOperand, 0..=2)
}

#[test]
fn increment_no_operand() {
    expect("++").to_error(MissingOperand, 0..=1)
}
#[test]
fn decrement_no_operand() {
    expect("--").to_error(MissingOperand, 0..=1)
}

//
// Test assignment to non-properties
//
#[test]
fn assign_non_field() {
    expect("1 = 1").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn assign_plus_non_field() {
    expect("1 += 1").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn assign_minus_non_field() {
    expect("1 -= 1").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn assign_times_non_field() {
    expect("1 *= 1").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn assign_divide_non_field() {
    expect("1 /= 1").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn increment_post_non_field() {
    expect("1++").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn decrement_post_non_field() {
    expect("1--").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn increment_pre_non_field() {
    expect("++1").to_error(AssignmentTargetMustBeIdentifier, 2)
}
#[test]
fn decrement_pre_non_field() {
    expect("--1").to_error(AssignmentTargetMustBeIdentifier, 2)
}

#[test]
fn assign_non_field_expr() {
    expect("1+2 = 1").to_error(AssignmentTargetMustBeIdentifier, 0..=2)
}
#[test]
fn assign_plus_non_field_expr() {
    expect("1+2 += 1").to_error(AssignmentTargetMustBeIdentifier, 0..=2)
}
#[test]
fn assign_minus_non_field_expr() {
    expect("1+2 -= 1").to_error(AssignmentTargetMustBeIdentifier, 0..=2)
}
#[test]
fn assign_times_non_field_expr() {
    expect("1+2 *= 1").to_error(AssignmentTargetMustBeIdentifier, 0..=2)
}
#[test]
fn assign_divide_non_field_expr() {
    expect("1+2 /= 1").to_error(AssignmentTargetMustBeIdentifier, 0..=2)
}
#[test]
fn increment_post_non_field_expr() {
    expect("(1+2)++").to_error(AssignmentTargetMustBeIdentifier, 0..=4)
}
#[test]
fn decrement_post_non_field_expr() {
    expect("(1+2)--").to_error(AssignmentTargetMustBeIdentifier, 0..=4)
}
#[test]
fn increment_pre_non_field_expr() {
    expect("++(1+2)").to_error(AssignmentTargetMustBeIdentifier, 2..=6)
}
#[test]
fn decrement_pre_non_field_expr() {
    expect("--(1+2)").to_error(AssignmentTargetMustBeIdentifier, 2..=6)
}

//
// Test that errors during the actual statement are propagated
//

#[test]
fn assign_error() {
    expect(":a = 1 + true").to_yield(tuple!())
}
#[test]
fn reassign_error() {
    expect(":a = 1; a  = 1 + true").to_yield(tuple!())
}
#[test]
fn assign_plus_error() {
    expect(":a = 1; a += 1 + true").to_yield(tuple!())
}
#[test]
fn assign_minus_error() {
    expect(":a = 1; a -= 1 + true").to_yield(tuple!())
}
#[test]
fn assign_times_error() {
    expect(":a = 1; a *= 1 + true").to_yield(tuple!())
}
#[test]
fn assign_divide_error() {
    expect(":a = 1; a /= 1 + true").to_yield(tuple!())
}
#[test]
fn assign_and_error() {
    expect(":a = true; a &&= true && 1").to_yield(tuple!())
}
#[test]
fn assign_or_error() {
    expect(":a = true; a ||= false && 1").to_yield(tuple!())
}

#[test]
fn assign_error_ref() {
    expect(":a = 1 + true; a").to_error(BadOperandType, 9..=12)
}
#[test]
fn reassign_error_ref() {
    expect(":a = 1; a  = 1 + true; a").to_error(BadOperandType, 17..=20)
}
#[test]
fn assign_plus_error_ref() {
    expect(":a = 1; a += 1 + true; a").to_error(BadOperandType, 17..=20)
}
#[test]
fn assign_minus_error_ref() {
    expect(":a = 1; a -= 1 + true; a").to_error(BadOperandType, 17..=20)
}
#[test]
fn assign_times_error_ref() {
    expect(":a = 1; a *= 1 + true; a").to_error(BadOperandType, 17..=20)
}
#[test]
fn assign_divide_error_ref() {
    expect(":a = 1; a /= 1 + true; a").to_error(BadOperandType, 17..=20)
}
#[test]
fn assign_and_error_ref() {
    expect(":a = true; a &&= true && 1; a").to_error(BadOperandType, 25)
}
#[test]
fn assign_or_error_ref() {
    expect(":a = true; a ||= false && 1; a").to_yield(true)
}

#[test]
fn assign_error_ref_twice() {
    expect(":a = 1 + true; a + a").to_error(BadOperandType, 9..=12)
}

//
// Test assignment to undefined values (ones that haven't been set)
//

#[test]
fn assign_undeclared() {
    expect("a = 1").to_yield(tuple!())
}
#[test]
fn assign_plus_undeclared() {
    expect("a += 1; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_minus_undeclared() {
    expect("a -= 1; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_times_undeclared() {
    expect("a *= 1; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_divide_undeclared() {
    expect("a /= 1; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_and_undeclared() {
    expect("a &&= true; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_or_undeclared() {
    expect("a ||= false; a").to_error(FieldNotSet, 0)
}
#[test]
fn increment_pre_undeclared() {
    expect("++a; a").to_error(FieldNotSet, 2)
}
#[test]
fn decrement_pre_undeclared() {
    expect("--a; a").to_error(FieldNotSet, 2)
}
#[test]
fn increment_post_undeclared() {
    expect("a++; a").to_error(FieldNotSet, 0)
}
#[test]
fn decrement_post_undeclared() {
    expect("a--; a").to_error(FieldNotSet, 0)
}

#[test]
fn assign_plus_undeclared_bad_type() {
    expect("a += true; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_minus_undeclared_bad_type() {
    expect("a -= true; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_times_undeclared_bad_type() {
    expect("a *= true; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_divide_undeclared_bad_type() {
    expect("a /= true; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_and_undeclared_bad_type() {
    expect("a &&= 1; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_or_undeclared_bad_type() {
    expect("a ||= 2; a").to_error(FieldNotSet, 0)
}

//
// Test assignment to undefined values (ones that haven't been set)
//

#[test]
fn assign_plus_undefined() {
    expect(":a; a += 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_undefined() {
    expect(":a; a -= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_times_undefined() {
    expect(":a; a *= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_undefined() {
    expect(":a; a /= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_undefined() {
    expect(":a; a &&= true").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_undefined() {
    expect(":a; a ||= false").to_error(FieldNotSet, 1)
}
#[test]
fn increment_pre_undefined() {
    expect(":a; ++a; a").to_error(FieldNotSet, 1)
}
#[test]
fn decrement_pre_undefined() {
    expect(":a; --a; a").to_error(FieldNotSet, 1)
}
#[test]
fn increment_post_undefined() {
    expect(":a; a++; a").to_error(FieldNotSet, 1)
}
#[test]
fn decrement_post_undefined() {
    expect(":a; a--; a").to_error(FieldNotSet, 1)
}

#[test]
fn assign_plus_undefined_bad_type() {
    expect(":a; a += true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_undefined_bad_type() {
    expect(":a; a -= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_times_undefined_bad_type() {
    expect(":a; a *= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_undefined_bad_type() {
    expect(":a; a /= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_undefined_bad_type() {
    expect(":a; a &&= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_error_bad_type() {
    expect(":a; a ||= 2; a").to_error(FieldNotSet, 1)
}

//
// Test behavior of assignment operations with declarations on the LHS
//

#[test]
fn assign_plus_declaration() {
    expect(":a += 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_declaration() {
    expect(":a -= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_times_declaration() {
    expect(":a *= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_declaration() {
    expect(":a /= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_declaration() {
    expect(":a &&= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_declaration() {
    expect(":a ||= false; a").to_error(FieldNotSet, 1)
}
#[test]
fn increment_pre_declaration() {
    expect("++:a; a").to_error(FieldNotSet, 3)
}
#[test]
fn decrement_pre_declaration() {
    expect("--:a; a").to_error(FieldNotSet, 3)
}
#[test]
fn increment_post_declaration() {
    expect(":a++; a").to_error(FieldNotSet, 1)
}
#[test]
fn decrement_post_declaration() {
    expect(":a--; a").to_error(FieldNotSet, 1)
}

#[test]
fn assign_plus_declaration_bad_type() {
    expect(":a += true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_declaration_bad_type() {
    expect(":a -= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_times_declaration_bad_type() {
    expect(":a *= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_declaration_bad_type() {
    expect(":a /= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_declaration_bad_type() {
    expect(":a &&= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_declaration_bad_type() {
    expect(":a ||= 2; a").to_error(FieldNotSet, 1)
}

//
// Test behavior of undefined self references
//

#[test]
fn assign_self_ref() {
    expect(":a = a + 1; a").to_error(FieldNotSet, 5)
}
#[test]
fn reassign_self_ref() {
    expect("a = a + 1; a").to_error(FieldNotSet, 4)
}
#[test]
fn assign_plus_self_ref() {
    expect("a += a; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_minus_self_ref() {
    expect("a -= a; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_multiply_self_ref() {
    expect("a *= a; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_divide_self_ref() {
    expect("a /= a; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_and_self_ref() {
    expect("a &&= a; a").to_error(FieldNotSet, 0)
}
#[test]
fn assign_or_self_ref() {
    expect("a ||= a; a").to_error(FieldNotSet, 0)
}

#[test]
fn assign_plus_declaration_self_ref() {
    expect(":a += a; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_declaration_self_ref() {
    expect(":a -= a; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_multiply_declaration_self_ref() {
    expect(":a *= a; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_declaration_self_ref() {
    expect(":a /= a; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_declaration_self_ref() {
    expect(":a &&= a; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_declaration_self_ref() {
    expect(":a ||= a; a").to_error(FieldNotSet, 1)
}

//
// Test bad type errors on assignment
//

#[test]
fn assign_plus_bad_type_left() {
    expect(":a = true; a += 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_plus_bad_type_right() {
    expect(":a = 2;    a += true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_plus_bad_type_both() {
    expect(":a = true; a += true; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_minus_bad_type_left() {
    expect(":a = true; a -= 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_minus_bad_type_right() {
    expect(":a = 2;    a -= true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_minus_bad_type_both() {
    expect(":a = true; a -= true; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_multiply_bad_type_left() {
    expect(":a = true; a *= 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_multiply_bad_type_right() {
    expect(":a = 2;    a *= true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_multiply_bad_type_both() {
    expect(":a = true; a *= true; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_divide_bad_type_left() {
    expect(":a = true; a /= 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_divide_bad_type_right() {
    expect(":a = 2;    a /= true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_divide_bad_type_both() {
    expect(":a = true; a /= true; a").to_error(UnsupportedOperator, 13..=14)
}

#[test]
fn assign_and_bad_type_left() {
    expect(":a = 2;    a &&= true; a").to_error(UnsupportedOperator, 13..=15)
}
#[test]
fn assign_and_bad_type_right() {
    expect(":a = true; a &&= 2; a").to_error(BadOperandType, 17)
}
#[test]
fn assign_and_bad_type_both() {
    expect(":a = 2;    a &&= 2; a").to_error(UnsupportedOperator, 13..=15)
}
#[test]
fn assign_or_bad_type_left() {
    expect(":a = 2;    a &&= true; a").to_error(UnsupportedOperator, 13..=15)
}
#[test]
fn assign_or_bad_type_right() {
    expect(":a = true; a &&= 2; a").to_error(BadOperandType, 17)
}
#[test]
fn assign_or_bad_type_both() {
    expect(":a = 2;    a &&= 2; a").to_error(UnsupportedOperator, 13..=15)
}

// Test error reporting (and result values) for references to failed assignments

#[test]
fn declaration_declaration_ref() {
    expect(":a; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_plus_declaration_ref() {
    expect(":a += 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_declaration_ref() {
    expect(":a -= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_times_declaration_ref() {
    expect(":a *= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_declaration_ref() {
    expect(":a /= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_declaration_ref() {
    expect(":a &&= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_declaration_ref() {
    expect(":a ||= false; a").to_error(FieldNotSet, 1)
}

#[test]
fn assign_plus_declaration_bad_type_ref() {
    expect(":a; a += true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_minus_declaration_bad_type_ref() {
    expect(":a; a -= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_times_declaration_bad_type_ref() {
    expect(":a; a *= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_divide_declaration_bad_type_ref() {
    expect(":a; a /= true; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_and_declaration_bad_type_ref() {
    expect(":a; a &&= 1; a").to_error(FieldNotSet, 1)
}
#[test]
fn assign_or_declaration_bad_type_ref() {
    expect(":a; a ||= 2; a").to_error(FieldNotSet, 1)
}

#[test]
fn assign_plus_bad_type_left_ref() {
    expect(":a = true; a += 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_plus_bad_type_right_ref() {
    expect(":a = 2;    a += true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_plus_bad_type_both_ref() {
    expect(":a = true; a += true; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_minus_bad_type_left_ref() {
    expect(":a = true; a -= 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_minus_bad_type_right_ref() {
    expect(":a = 2;    a -= true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_minus_bad_type_both_ref() {
    expect(":a = true; a -= true; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_multiply_bad_type_left_ref() {
    expect(":a = true; a *= 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_multiply_bad_type_right_ref() {
    expect(":a = 2;    a *= true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_multiply_bad_type_both_ref() {
    expect(":a = true; a *= true; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_divide_bad_type_left_ref() {
    expect(":a = true; a /= 2; a").to_error(UnsupportedOperator, 13..=14)
}
#[test]
fn assign_divide_bad_type_right_ref() {
    expect(":a = 2;    a /= true; a").to_error(BadOperandType, 16..=19)
}
#[test]
fn assign_divide_bad_type_both_ref() {
    expect(":a = true; a /= true; a").to_error(UnsupportedOperator, 13..=14)
}

#[test]
fn assign_and_bad_type_left_ref() {
    expect(":a = 2;    a &&= true; a").to_error(UnsupportedOperator, 13..=15)
}
#[test]
fn assign_and_bad_type_right_ref() {
    expect(":a = true; a &&= 2; a").to_error(BadOperandType, 17)
}
#[test]
fn assign_and_bad_type_both_ref() {
    expect(":a = 2;    a &&= 2; a").to_error(UnsupportedOperator, 13..=15)
}
#[test]
fn assign_or_bad_type_left_ref() {
    expect(":a = 2;    a &&= true; a").to_error(UnsupportedOperator, 13..=15)
}
#[test]
fn assign_or_bad_type_right_ref() {
    expect(":a = true; a &&= 2; a").to_error(BadOperandType, 17)
}
#[test]
fn assign_or_bad_type_both_ref() {
    expect(":a = 2;    a &&= 2; a").to_error(UnsupportedOperator, 13..=15)
}
