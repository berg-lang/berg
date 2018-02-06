#![recursion_limit = "512"]
#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    //
    // Test declarations with references
    //

    assign_ref:        ":a = 1; a" => value(1),
    reassign_ref:      ":a = 1; a = 2; a" => value(2),
    two_fields_ref: ":a = 1; :b = 2; a + b" => value(3),

    assign_plus_ref:   ":a = 1;  a += 2; a" => value(3),
    assign_minus_ref:  ":a = 3;  a -= 2; a" => value(1),
    assign_times_ref:  ":a = 2;  a *= 3; a" => value(6),
    assign_divide_ref: ":a = 12; a /= 4; a" => value(3),
    increment_post_ref: ":a = 1; a++; a" => value(2),
    increment_pre_ref: ":a = 1; ++a; a" => value(2),
    decrement_post_ref: ":a = 1; a--; a" => value(0),
    decrement_pre_ref: ":a = 1; --a; a" => value(0),

    assign_and_true_true_ref:   ":a = true;  a &&= true;  a" => value(true),
    assign_and_true_false_ref:  ":a = true;  a &&= false; a" => value(false),
    assign_and_false_true_ref:  ":a = false; a &&= true;  a" => value(false),
    assign_and_false_false_ref: ":a = false; a &&= false; a" => value(false),
    assign_or_true_true_ref:    ":a = true;  a ||= true;  a" => value(true),
    assign_or_true_false_ref:   ":a = true;  a ||= false; a" => value(true),
    assign_or_false_true_ref:   ":a = false; a ||= true;  a" => value(true),
    assign_or_false_false_ref:  ":a = false; a ||= false; a" => value(false),

    //
    // Test precedence
    //
    assign_precedence: "a = false; b = false || true && 14 == 2 + 3 * 4; b" => value(true),
    assign_plus_precedence: ":b = 1; b += 2 + 3 * 4; b" => value(15),
    assign_minus_precedence: ":b = 1; b -= 2 + 3 * 4; b" => value(-13),
    assign_multiply_precedence: ":b = 2; b *= 2 + 3 * 4; b" => value(28),
    assign_divide_precedence: ":b = 28; b /= 2 + 3 * 4; b" => value(2),
    assign_and_precedence: "a = false; b = true; b &&= false || true && 14 == 2 + 3 * 4; b" => value(true),
    assign_or_precedence: "a = false; b = false; b ||= false || true && 14 == 2 + 3 * 4; b" => value(true),

    //
    // Test declarations without references
    //

    bare_declaration: ":a"               => error(FieldNotSet@1),
    assign:         ":a = 1"             => value(Nothing),
    reassign:       ":a = 1;  a = 2"  => value(Nothing),

    assign_plus:    ":a = 1;  a += 2" => value(Nothing),
    assign_minus:   ":a = 3;  a -= 2" => value(Nothing),
    assign_times:   ":a = 2;  a *= 3" => value(Nothing),
    assign_divide:  ":a = 12; a /= 4" => value(Nothing),
    assign_and:     ":a = true;  a &&= false" => value(Nothing),
    assign_or:      ":a = false; a ||= true"  => value(Nothing),
    increment_post: ":a = 1;  a++"    => value(Nothing),
    increment_pre:  ":a = 1;  ++a"    => value(Nothing),
    decrement_post: ":a = 1;  a--"    => value(Nothing),
    decrement_pre:  ":a = 1;  --a"    => value(Nothing),

    //
    // Test behavior of references to other fields of the same name
    //

    assign_prev_ref:         ":a = 1; :a = a + 1; a" => value(2),
    reassign_prev_ref:       ":a = 1; a = a + 1; a" => value(2),

    assign_plus_prev_ref:    ":a = 3;  a += a; a" => value(6),
    assign_minus_prev_ref:   ":a = 3;  a -= a; a" => value(0),
    assign_times_prev_ref:   ":a = 3;  a *= a; a" => value(9),
    assign_divide_prev_ref:  ":a = 3; a /= a; a" => value(1),
    assign_and_prev_ref_true:  ":a = true;  a &&= a; a" => value(true),
    assign_and_prev_ref_false: ":a = false; a &&= a; a" => value(false),
    assign_or_prev_ref_true:   ":a = true; a ||= a; a"  => value(true),
    assign_or_prev_ref_false:  ":a = false; a ||= a; a"  => value(false),

    //
    // Test missing syntax
    //
    assign_missing_right: ":a = ; a" => error(MissingOperand@3),
    reassign_missing_right: "a = ; a" => error(MissingOperand@2),
    assign_plus_missing_right: ":a += ; a" => error(MissingOperand@[3-4]),
    assign_minus_missing_right: ":a -= ; a" => error(MissingOperand@[3-4]),
    assign_multiply_missing_right: ":a *= ; a" => error(MissingOperand@[3-4]),
    assign_divide_missing_right: ":a /= ; a" => error(MissingOperand@[3-4]),
    assign_and_missing_right: ":a &&= ; a" => error(MissingOperand@[3-5]),
    assign_or_missing_right: ":a ||= ; a" => error(MissingOperand@[3-5]),

    assign_missing_left: "= 1" => error(MissingOperand@0),
    assign_plus_missing_left: "+= 1" => error(MissingOperand@[0-1]),
    assign_minus_missing_left: "-= 1" => error(MissingOperand@[0-1]),
    assign_multiply_missing_left: "*= 1" => error(MissingOperand@[0-1]),
    assign_divide_missing_left: "/= 1" => error(MissingOperand@[0-1]),
    assign_and_missing_left: "&&= false" => error(MissingOperand@[0-2]),
    assign_or_missing_left: "||= true" => error(MissingOperand@[0-2]),

    assign_missing_both: "=" => error(MissingOperand@0),
    assign_plus_missing_both: "+=" => error(MissingOperand@[0-1]),
    assign_minus_missing_both: "-=" => error(MissingOperand@[0-1]),
    assign_multiply_missing_both: "*=" => error(MissingOperand@[0-1]),
    assign_divide_missing_both: "/=" => error(MissingOperand@[0-1]),
    assign_and_missing_both: "&&=" => error(MissingOperand@[0-2]),
    assign_or_missing_both: "||=" => error(MissingOperand@[0-2]),

    increment_no_operand: "++" => error(MissingOperand@[0-1]),
    decrement_no_operand: "--" => error(MissingOperand@[0-1]),

    //
    // Test assignment to non-properties
    //
    assign_non_field:         "1 = 1" => error(AssignmentTargetMustBeIdentifier@0),
    assign_plus_non_field:    "1 += 1" => error(AssignmentTargetMustBeIdentifier@0),
    assign_minus_non_field:   "1 -= 1" => error(AssignmentTargetMustBeIdentifier@0),
    assign_times_non_field:   "1 *= 1" => error(AssignmentTargetMustBeIdentifier@0),
    assign_divide_non_field:  "1 /= 1" => error(AssignmentTargetMustBeIdentifier@0),
    increment_post_non_field: "1++" => error(AssignmentTargetMustBeIdentifier@0),
    decrement_post_non_field: "1--" => error(AssignmentTargetMustBeIdentifier@0),
    increment_pre_non_field:  "++1" => error(AssignmentTargetMustBeIdentifier@2),
    decrement_pre_non_field:  "--1" => error(AssignmentTargetMustBeIdentifier@2),

    assign_non_field_expr:         "1+2 = 1" => error(AssignmentTargetMustBeIdentifier@[0-2]),
    assign_plus_non_field_expr:    "1+2 += 1" => error(AssignmentTargetMustBeIdentifier@[0-2]),
    assign_minus_non_field_expr:   "1+2 -= 1" => error(AssignmentTargetMustBeIdentifier@[0-2]),
    assign_times_non_field_expr:   "1+2 *= 1" => error(AssignmentTargetMustBeIdentifier@[0-2]),
    assign_divide_non_field_expr:  "1+2 /= 1" => error(AssignmentTargetMustBeIdentifier@[0-2]),
    increment_post_non_field_expr: "(1+2)++" => error(AssignmentTargetMustBeIdentifier@[1-3]),
    decrement_post_non_field_expr: "(1+2)--" => error(AssignmentTargetMustBeIdentifier@[1-3]),
    increment_pre_non_field_expr:  "++(1+2)" => error(AssignmentTargetMustBeIdentifier@[3-5]),
    decrement_pre_non_field_expr:  "--(1+2)" => error(AssignmentTargetMustBeIdentifier@[3-5]),

    //
    // Test that errors during the actual statement are propagated
    //

    assign_error:        ":a = 1 + true" => value(Nothing),
    reassign_error:      ":a = 1; a  = 1 + true"    => value(Nothing),
    assign_plus_error:   ":a = 1; a += 1 + true"    => value(Nothing),
    assign_minus_error:  ":a = 1; a -= 1 + true"    => value(Nothing),
    assign_times_error:  ":a = 1; a *= 1 + true"    => value(Nothing),
    assign_divide_error: ":a = 1; a /= 1 + true"    => value(Nothing),
    assign_and_error:    ":a = true; a &&= true && 1"  => value(Nothing),
    assign_or_error:     ":a = true; a ||= false && 1" => value(Nothing),

    assign_error_ref:        ":a = 1 + true; a" => error(BadType@[9-12]),
    reassign_error_ref:      ":a = 1; a  = 1 + true; a"    => error(BadType@[17-20]),
    assign_plus_error_ref:   ":a = 1; a += 1 + true; a"    => error(BadType@[17-20]),
    assign_minus_error_ref:  ":a = 1; a -= 1 + true; a"    => error(BadType@[17-20]),
    assign_times_error_ref:  ":a = 1; a *= 1 + true; a"    => error(BadType@[17-20]),
    assign_divide_error_ref: ":a = 1; a /= 1 + true; a"    => error(BadType@[17-20]),
    assign_and_error_ref:    ":a = true; a &&= true && 1; a"  => error(BadType@25),
    assign_or_error_ref:     ":a = true; a ||= false && 1; a" => value(true),

    assign_error_ref_twice: ":a = 1 + true; a + a; a" => error(BadType@[9-12]),

    //
    // Test assignment to undefined values (ones that haven't been set)
    //

    assign_undeclared:        "a = 1"       => value(Nothing),
    assign_plus_undeclared:   "a += 1; a"      => error(FieldNotSet@0),
    assign_minus_undeclared:  "a -= 1; a"      => error(FieldNotSet@0),
    assign_times_undeclared:  "a *= 1; a"      => error(FieldNotSet@0),
    assign_divide_undeclared: "a /= 1; a"      => error(FieldNotSet@0),
    assign_and_undeclared:    "a &&= true; a"  => error(FieldNotSet@0),
    assign_or_undeclared:     "a ||= false; a" => error(FieldNotSet@0),
    increment_pre_undeclared: "++a; a"  => error(FieldNotSet@2),
    decrement_pre_undeclared: "--a; a"  => error(FieldNotSet@2),
    increment_post_undeclared: "a++; a" => error(FieldNotSet@0),
    decrement_post_undeclared: "a--; a" => error(FieldNotSet@0),

    assign_plus_undeclared_bad_type:   "a += true; a" => error(FieldNotSet@0),
    assign_minus_undeclared_bad_type:  "a -= true; a" => error(FieldNotSet@0),
    assign_times_undeclared_bad_type:  "a *= true; a" => error(FieldNotSet@0),
    assign_divide_undeclared_bad_type: "a /= true; a" => error(FieldNotSet@0),
    assign_and_undeclared_bad_type:    "a &&= 1; a"   => error(FieldNotSet@0),
    assign_or_undeclared_bad_type:     "a ||= 2; a"   => error(FieldNotSet@0),

    //
    // Test assignment to undefined values (ones that haven't been set)
    //

    assign_plus_undefined:   ":a; a += 1; a"      => error(FieldNotSet@1),
    assign_minus_undefined:  ":a; a -= 1; a"      => error(FieldNotSet@1),
    assign_times_undefined:  ":a; a *= 1; a"      => error(FieldNotSet@1),
    assign_divide_undefined: ":a; a /= 1; a"      => error(FieldNotSet@1),
    assign_and_undefined:    ":a; a &&= true"  => error(FieldNotSet@1),
    assign_or_undefined:     ":a; a ||= false" => error(FieldNotSet@1),
    increment_pre_undefined:  ":a; ++a; a"  => error(FieldNotSet@1),
    decrement_pre_undefined:  ":a; --a; a"  => error(FieldNotSet@1),
    increment_post_undefined: ":a; a++; a" => error(FieldNotSet@1),
    decrement_post_undefined: ":a; a--; a" => error(FieldNotSet@1),

    assign_plus_undefined_bad_type:   ":a; a += true; a" => error(FieldNotSet@1),
    assign_minus_undefined_bad_type:  ":a; a -= true; a" => error(FieldNotSet@1),
    assign_times_undefined_bad_type:  ":a; a *= true; a" => error(FieldNotSet@1),
    assign_divide_undefined_bad_type: ":a; a /= true; a" => error(FieldNotSet@1),
    assign_and_undefined_bad_type:    ":a; a &&= 1; a"   => error(FieldNotSet@1),
    assign_or_error_bad_type:     ":a; a ||= 2; a"   => error(FieldNotSet@1),

    //
    // Test behavior of assignment operations with declarations on the LHS
    //

    assign_plus_declaration:   ":a += 1; a"      => error(FieldNotSet@1),
    assign_minus_declaration:  ":a -= 1; a"      => error(FieldNotSet@1),
    assign_times_declaration:  ":a *= 1; a"      => error(FieldNotSet@1),
    assign_divide_declaration: ":a /= 1; a"      => error(FieldNotSet@1),
    assign_and_declaration:    ":a &&= true; a"  => error(FieldNotSet@1),
    assign_or_declaration:     ":a ||= false; a" => error(FieldNotSet@1),
    increment_pre_declaration: "++:a; a"  => error(FieldNotSet@3),
    decrement_pre_declaration: "--:a; a"  => error(FieldNotSet@3),
    increment_post_declaration: ":a++; a" => error(FieldNotSet@1),
    decrement_post_declaration: ":a--; a" => error(FieldNotSet@1),

    assign_plus_declaration_bad_type:   ":a += true; a" => error(FieldNotSet@1),
    assign_minus_declaration_bad_type:  ":a -= true; a" => error(FieldNotSet@1),
    assign_times_declaration_bad_type:  ":a *= true; a" => error(FieldNotSet@1),
    assign_divide_declaration_bad_type: ":a /= true; a" => error(FieldNotSet@1),
    assign_and_declaration_bad_type:    ":a &&= 1; a"   => error(FieldNotSet@1),
    assign_or_declaration_bad_type:     ":a ||= 2; a"   => error(FieldNotSet@1),

    //
    // Test behavior of undefined self references
    //

    assign_self_ref:          ":a = a + 1; a" => error(FieldNotSet@5),
    reassign_self_ref:        "a = a + 1; a" => error(FieldNotSet@4),
    assign_plus_self_ref:     "a += a; a" => error(FieldNotSet@0),
    assign_minus_self_ref:    "a -= a; a" => error(FieldNotSet@0),
    assign_multiply_self_ref: "a *= a; a" => error(FieldNotSet@0),
    assign_divide_self_ref:   "a /= a; a" => error(FieldNotSet@0),
    assign_and_self_ref:      "a &&= a; a" => error(FieldNotSet@0),
    assign_or_self_ref:       "a ||= a; a" => error(FieldNotSet@0),

    assign_plus_declaration_self_ref:     ":a += a; a" => error(FieldNotSet@1),
    assign_minus_declaration_self_ref:    ":a -= a; a" => error(FieldNotSet@1),
    assign_multiply_declaration_self_ref: ":a *= a; a" => error(FieldNotSet@1),
    assign_divide_declaration_self_ref:   ":a /= a; a" => error(FieldNotSet@1),
    assign_and_declaration_self_ref:      ":a &&= a; a" => error(FieldNotSet@1),
    assign_or_declaration_self_ref:       ":a ||= a; a" => error(FieldNotSet@1),

    //
    // Test bad type errors on assignment
    //

    assign_plus_bad_type_left:      ":a = true; a += 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_plus_bad_type_right:     ":a = 2;    a += true; a" => error(BadType@[16-19]),
    assign_plus_bad_type_both:      ":a = true; a += true; a" => error(UnsupportedOperator@[13-14]),
    assign_minus_bad_type_left:     ":a = true; a -= 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_minus_bad_type_right:    ":a = 2;    a -= true; a" => error(BadType@[16-19]),
    assign_minus_bad_type_both:     ":a = true; a -= true; a" => error(UnsupportedOperator@[13-14]),
    assign_multiply_bad_type_left:  ":a = true; a *= 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_multiply_bad_type_right: ":a = 2;    a *= true; a" => error(BadType@[16-19]),
    assign_multiply_bad_type_both:  ":a = true; a *= true; a" => error(UnsupportedOperator@[13-14]),
    assign_divide_bad_type_left:    ":a = true; a /= 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_divide_bad_type_right:   ":a = 2;    a /= true; a" => error(BadType@[16-19]),
    assign_divide_bad_type_both:    ":a = true; a /= true; a" => error(UnsupportedOperator@[13-14]),

    assign_and_bad_type_left:       ":a = 2;    a &&= true; a" => error(UnsupportedOperator@[13-15]),
    assign_and_bad_type_right:      ":a = true; a &&= 2; a"    => error(BadType@17),
    assign_and_bad_type_both:       ":a = 2;    a &&= 2; a"    => error(UnsupportedOperator@[13-15]),
    assign_or_bad_type_left:        ":a = 2;    a &&= true; a" => error(UnsupportedOperator@[13-15]),
    assign_or_bad_type_right:       ":a = true; a &&= 2; a"    => error(BadType@17),
    assign_or_bad_type_both:        ":a = 2;    a &&= 2; a"    => error(UnsupportedOperator@[13-15]),

    // Test error reporting (and result values) for references to failed assignments

    declaration_declaration_ref:   ":a; a"           => error(FieldNotSet@1),
    assign_plus_declaration_ref:   ":a += 1; a"      => error(FieldNotSet@1),
    assign_minus_declaration_ref:  ":a -= 1; a"      => error(FieldNotSet@1),
    assign_times_declaration_ref:  ":a *= 1; a"      => error(FieldNotSet@1),
    assign_divide_declaration_ref: ":a /= 1; a"      => error(FieldNotSet@1),
    assign_and_declaration_ref:    ":a &&= true; a"  => error(FieldNotSet@1),
    assign_or_declaration_ref:     ":a ||= false; a" => error(FieldNotSet@1),

    assign_plus_declaration_bad_type_ref:   ":a; a += true; a" => error(FieldNotSet@1),
    assign_minus_declaration_bad_type_ref:  ":a; a -= true; a" => error(FieldNotSet@1),
    assign_times_declaration_bad_type_ref:  ":a; a *= true; a" => error(FieldNotSet@1),
    assign_divide_declaration_bad_type_ref: ":a; a /= true; a" => error(FieldNotSet@1),
    assign_and_declaration_bad_type_ref:    ":a; a &&= 1; a"   => error(FieldNotSet@1),
    assign_or_declaration_bad_type_ref:     ":a; a ||= 2; a"   => error(FieldNotSet@1),

    assign_plus_bad_type_left_ref:      ":a = true; a += 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_plus_bad_type_right_ref:     ":a = 2;    a += true; a" => error(BadType@[16-19]),
    assign_plus_bad_type_both_ref:      ":a = true; a += true; a" => error(UnsupportedOperator@[13-14]),
    assign_minus_bad_type_left_ref:     ":a = true; a -= 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_minus_bad_type_right_ref:    ":a = 2;    a -= true; a" => error(BadType@[16-19]),
    assign_minus_bad_type_both_ref:     ":a = true; a -= true; a" => error(UnsupportedOperator@[13-14]),
    assign_multiply_bad_type_left_ref:  ":a = true; a *= 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_multiply_bad_type_right_ref: ":a = 2;    a *= true; a" => error(BadType@[16-19]),
    assign_multiply_bad_type_both_ref:  ":a = true; a *= true; a" => error(UnsupportedOperator@[13-14]),
    assign_divide_bad_type_left_ref:    ":a = true; a /= 2; a"    => error(UnsupportedOperator@[13-14]),
    assign_divide_bad_type_right_ref:   ":a = 2;    a /= true; a" => error(BadType@[16-19]),
    assign_divide_bad_type_both_ref:    ":a = true; a /= true; a" => error(UnsupportedOperator@[13-14]),

    assign_and_bad_type_left_ref:       ":a = 2;    a &&= true; a" => error(UnsupportedOperator@[13-15]),
    assign_and_bad_type_right_ref:      ":a = true; a &&= 2; a"    => error(BadType@17),
    assign_and_bad_type_both_ref:       ":a = 2;    a &&= 2; a"    => error(UnsupportedOperator@[13-15]),
    assign_or_bad_type_left_ref:        ":a = 2;    a &&= true; a" => error(UnsupportedOperator@[13-15]),
    assign_or_bad_type_right_ref:       ":a = true; a &&= 2; a"    => error(BadType@17),
    assign_or_bad_type_both_ref:        ":a = 2;    a &&= 2; a"    => error(UnsupportedOperator@[13-15]),
}
