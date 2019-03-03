use crate::*;

// Declaration without reference (sets value, returns Nothing)
#[test]
fn declare() {
    expect("a: 1").to_yield(tuple!())
}

// Test declare laziness
#[test]
fn declare_lazy() {
    expect("a = 1; b = 2; c: a + b; a++; b++; c").to_yield(5)
}
#[test]
fn declare_scope() {
    expect("a = 1; c: :a; c").to_error(FieldNotSet, 11)
}

//
// Test declarations with references
//

#[test]
fn declare_ref() {
    expect("a: 1; a").to_yield(1)
}
#[test]
fn declare_two_fields_ref() {
    expect("a: 1; b: 2; a + b").to_yield(3)
}

// #[test]
// fn redeclare_error()                  { expect( "a: 1; a: 2"          ).to_error(ImmutableField,6) }
// #[test]
// fn declare_reassign_error()           { expect( "a: 1; a = 2"         ).to_error(ImmutableField,6) }
// #[test]
// fn declare_prev_ref_error()           { expect( "a: 1; a: a + 1; a"   ).to_error(ImmutableField,6) }
// #[test]
// fn redeclare_plus_error()             { expect( "a: 1; a += 2"        ).to_error(ImmutableField,6) }
// #[test]
// fn redeclare_minus_error()            { expect( "a: 1; a -= 2"        ).to_error(ImmutableField,6) }
// #[test]
// fn redeclare_times_error()            { expect( "a: 1; a *= 2"        ).to_error(ImmutableField,6) }
// #[test]
// fn redeclare_divide_error()           { expect( "a: 1; a /= 2"        ).to_error(ImmutableField,6) }
// #[test]
// fn redeclare_and_error()              { expect( "a: true; a ||= true" ).to_error(ImmutableField,9) }
// #[test]
// fn redeclare_or_error()               { expect( "a: true; a &&= true" ).to_error(ImmutableField,9) }
// #[test]
// fn increment_post_error()             { expect( "a: 1; a--"           ).to_error(ImmutableField,6) }
// #[test]
// fn increment_pre_error()              { expect( "a: 1; --a"           ).to_error(ImmutableField,8) }
// #[test]
// fn decrement_post_error()             { expect( "a: 1; a--"           ).to_error(ImmutableField,6) }
// #[test]
// fn decrement_pre_error()              { expect( "a: 1; --a"           ).to_error(ImmutableField,8) }

// #[test]
// fn redeclare_plus_multiple_errors()   { expect( "a: true; a += true"  ).to_error(ImmutableField,9) }
// #[test]
// fn redeclare_minus_multiple_errors()  { expect( "a: true; a -= true"  ).to_error(ImmutableField,9) }
// #[test]
// fn redeclare_times_multiple_errors()  { expect( "a: true; a *= true"  ).to_error(ImmutableField,9) }
// #[test]
// fn redeclare_divide_multiple_errors() { expect( "a: true; a /= true"  ).to_error(ImmutableField,9) }
// #[test]
// fn redeclare_and_multiple_errors()    { expect( "a: 1; a ||= 2"       ).to_error(ImmutableField,9) }
// #[test]
// fn redeclare_or_multiple_errors()     { expect( "a: 1; a &&= 2"       ).to_error(ImmutableField,9) }
// #[test]
// fn increment_post_multiple_errors()   { expect( "a: true; a--"        ).to_error(ImmutableField,9) }
// #[test]
// fn increment_pre_multiple_errors()    { expect( "a: true; --a"        ).to_error(ImmutableField,11) }
// #[test]
// fn decrement_post_multiple_errors()   { expect( "a: true; a--"        ).to_error(ImmutableField,9) }
// #[test]
// fn decrement_pre_multiple_errors()    { expect( "a: true; --a"        ).to_error(ImmutableField,11) }

//
// Test precedence
//
#[test]
fn declare_precedence() {
    expect("a: false; b: false || true && 14 == 2 + 3 * 4; b").to_yield(true)
}

//
// Test missing syntax
//
#[test]
fn declare_missing_right() {
    expect("a: ; a").to_error(MissingOperand, 1)
}
#[test]
fn declare_missing_left() {
    expect(": 1").to_error(MissingOperand, 0)
}
#[test]
fn declare_missing_both() {
    expect(":").to_error(MissingOperand, 0)
}

//
// Test assignment to non-properties
//
#[test]
fn declare_non_field() {
    expect("1: 1").to_error(AssignmentTargetMustBeIdentifier, 0)
}
#[test]
fn declare_non_field_expr() {
    expect("1+2: 1").to_error(AssignmentTargetMustBeIdentifier, 0..=2)
}

//
// Test that errors during the actual statement are propagated
//

#[test]
fn declare_error() {
    expect("a: 1 + true").to_yield(tuple!())
}
#[test]
fn declare_error_ref() {
    expect("a: 1 + true; a").to_error(BadOperandType, 7..=10)
}
#[test]
fn declare_error_ref_twice() {
    expect("a: 1 + true; a + a").to_error(BadOperandType, 7..=10)
}

//
// Test behavior of undefined self references
//

#[test]
fn declare_self_ref() {
    expect("a: a + 1; a").to_error(CircularDependency, 3..=7)
}
