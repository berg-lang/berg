#![recursion_limit = "512"]
#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    // Declaration without reference (sets value, returns Nothing)
    declare:   "a: 1"          => value(Nothing),

    // Test declare laziness
    declare_lazy: "a = 1; b = 2; c: a + b; a++; b++; c" => value(5),
    declare_scope: "a = 1; c: :a; c" => error(FieldNotSet@11),

    //
    // Test declarations with references
    //

    declare_ref:        "a: 1; a" => value(1),
    declare_two_fields_ref: "a: 1; b: 2; a + b" => value(3),

    // redeclare_error: "a: 1; a: 2" => error(ImmutableField@6),
    // declare_reassign_error:  "a: 1; a = 2" => error(ImmutableField@6),
    // declare_prev_ref_error:        "a: 1; a: a + 1; a" => error(ImmutableField@6),
    // redeclare_plus_error: "a: 1; a += 2" => error(ImmutableField@6),
    // redeclare_minus_error: "a: 1; a -= 2" => error(ImmutableField@6),
    // redeclare_times_error: "a: 1; a *= 2" => error(ImmutableField@6),
    // redeclare_divide_error: "a: 1; a /= 2" => error(ImmutableField@6),
    // redeclare_and_error: "a: true; a ||= true" => error(ImmutableField@9),
    // redeclare_or_error: "a: true; a &&= true" => error(ImmutableField@9),
    // increment_post_error: "a: 1; a--" => error(ImmutableField@6),
    // increment_pre_error: "a: 1; --a" => error(ImmutableField@8),
    // decrement_post_error: "a: 1; a--" => error(ImmutableField@6),
    // decrement_pre_error: "a: 1; --a" => error(ImmutableField@8),

    // redeclare_plus_multiple_errors: "a: true; a += true" => error(ImmutableField@9),
    // redeclare_minus_multiple_errors: "a: true; a -= true" => error(ImmutableField@9),
    // redeclare_times_multiple_errors: "a: true; a *= true" => error(ImmutableField@9),
    // redeclare_divide_multiple_errors: "a: true; a /= true" => error(ImmutableField@9),
    // redeclare_and_multiple_errors: "a: 1; a ||= 2" => error(ImmutableField@9),
    // redeclare_or_multiple_errors: "a: 1; a &&= 2" => error(ImmutableField@9),
    // increment_post_multiple_errors: "a: true; a--" => error(ImmutableField@9),
    // increment_pre_multiple_errors: "a: true; --a" => error(ImmutableField@11),
    // decrement_post_multiple_errors: "a: true; a--" => error(ImmutableField@9),
    // decrement_pre_multiple_errors: "a: true; --a" => error(ImmutableField@11),

    //
    // Test precedence
    //
    declare_precedence: "a: false; b: false || true && 14 == 2 + 3 * 4; b" => value(true),

    //
    // Test missing syntax
    //
    declare_missing_right:   "a: ; a" => error(MissingOperand@1),
    declare_missing_left:    ": 1" => error(MissingOperand@0),
    declare_missing_both:    ":" => error(MissingOperand@0),

    //
    // Test assignment to non-properties
    //
    declare_non_field:       "1: 1" => error(AssignmentTargetMustBeIdentifier@0),
    declare_non_field_expr:  "1+2: 1" => error(AssignmentTargetMustBeIdentifier@[0-2]),

    //
    // Test that errors during the actual statement are propagated
    //

    declare_error:           "a: 1 + true" => value(Nothing),
    declare_error_ref:       "a: 1 + true; a" => error(BadType@[7-10]),
    declare_error_ref_twice: "a: 1 + true; a + a" => error(BadType@[7-10]),

    //
    // Test behavior of undefined self references
    //

    declare_self_ref:          "a: a + 1; a" => error(CircularDependency@[3-7]),
}
