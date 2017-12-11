#[macro_use]
pub mod compiler_test;

compiler_tests! {
    //
    // Test declarations with references
    //

    assign_ref:        ":a = 1; a" => type(1),
    reassign_ref:      ":a = 1; a = 2; a" => type(2),
    two_variables_ref: ":a = 1; :b = 2; a + b" => type(3),

    assign_plus_ref:   ":a = 1;  a += 2; a" => type(3),
    assign_minus_ref:  ":a = 3;  a -= 2; a" => type(1),
    assign_times_ref:  ":a = 2;  a *= 3; a" => type(6),
    assign_divide_ref: ":a = 12; a /= 4; a" => type(3),
    increment_post_ref: ":a = 1; a++; a" => type(2),
    increment_pre_ref: ":a = 1; ++a; a" => type(2),
    decrement_post_ref: ":a = 1; a--; a" => type(0),
    decrement_pre_ref: ":a = 1; --a; a" => type(0),

    assign_and_true_true_ref:   ":a = true;  a &&= true;  a" => type(true),
    assign_and_true_false_ref:  ":a = true;  a &&= false; a" => type(false),
    assign_and_false_true_ref:  ":a = false; a &&= true;  a" => type(false),
    assign_and_false_false_ref: ":a = false; a &&= false; a" => type(false),
    assign_or_true_true_ref:    ":a = true;  a ||= true;  a" => type(true),
    assign_or_true_false_ref:   ":a = true;  a ||= false; a" => type(true),
    assign_or_false_true_ref:   ":a = false; a ||= true;  a" => type(true),
    assign_or_false_false_ref:  ":a = false; a ||= false; a" => type(false),

    //
    // Test precedence
    //
    assign_precedence: "a = false; b = false || true && 14 == 2 + 3 * 4; b" => type(true),
    assign_plus_precedence: ":b = 1; b += 2 + 3 * 4; b" => type(15),
    assign_minus_precedence: ":b = 1; b -= 2 + 3 * 4; b" => type(-13),
    assign_multiply_precedence: ":b = 2; b *= 2 + 3 * 4; b" => type(28),
    assign_divide_precedence: ":b = 28; b /= 2 + 3 * 4; b" => type(2),
    assign_and_precedence: "a = false; b = true; b &&= false || true && 14 == 2 + 3 * 4; b" => type(true),
    assign_or_precedence: "a = false; b = false; b ||= false || true && 14 == 2 + 3 * 4; b" => type(true),

    //
    // Test declarations without references
    //

    bare_declaration: ":a"               => type(error) errors(FieldNotSet@1),
    assign:         ":a = 1"             => type(nothing),
    reassign:       ":a = 1;  a = 2"  => type(nothing),

    assign_plus:    ":a = 1;  a += 2" => type(nothing),
    assign_minus:   ":a = 3;  a -= 2" => type(nothing),
    assign_times:   ":a = 2;  a *= 3" => type(nothing),
    assign_divide:  ":a = 12; a /= 4" => type(nothing),
    assign_and:     ":a = true;  a &&= false" => type(nothing),
    assign_or:      ":a = false; a ||= true"  => type(nothing),
    increment_post: ":a = 1;  a++"    => type(nothing),
    increment_pre:  ":a = 1;  ++a"    => type(nothing),
    decrement_post: ":a = 1;  a--"    => type(nothing),
    decrement_pre:  ":a = 1;  --a"    => type(nothing),

    //
    // Test behavior of references to other variables of the same name
    //

    assign_prev_ref:         ":a = 1; :a = a + 1; a" => type(2),
    reassign_prev_ref:       ":a = 1; a = a + 1; a" => type(2),

    assign_plus_prev_ref:    ":a = 3;  a += a; a" => type(6),
    assign_minus_prev_ref:   ":a = 3;  a -= a; a" => type(0),
    assign_times_prev_ref:   ":a = 3;  a *= a; a" => type(9),
    assign_divide_prev_ref:  ":a = 3; a /= a; a" => type(1),
    assign_and_prev_ref_true:  ":a = true;  a &&= a; a" => type(true),
    assign_and_prev_ref_false: ":a = false; a &&= a; a" => type(false),
    assign_or_prev_ref_true:   ":a = true; a ||= a; a"  => type(true),
    assign_or_prev_ref_false:  ":a = false; a ||= a; a"  => type(false),

    //
    // Test missing syntax
    //
    assign_missing_right: ":a =" => type(nothing) errors(MissingRightOperand@3),
    reassign_missing_right: "a =" => type(nothing) errors(MissingRightOperand@2),
    assign_plus_missing_right: "a +=" => type(nothing) errors(NoSuchField@0,MissingRightOperand@[2-3]),
    assign_minus_missing_right: "a -=" => type(nothing) errors(NoSuchField@0,MissingRightOperand@[2-3]),
    assign_multiply_missing_right: "a *=" => type(nothing) errors(NoSuchField@0,MissingRightOperand@[2-3]),
    assign_divide_missing_right: "a /=" => type(nothing) errors(NoSuchField@0,MissingRightOperand@[2-3]),
    assign_and_missing_right: "a &&=" => type(nothing) errors(NoSuchField@0,MissingRightOperand@[2-4]),
    assign_or_missing_right: "a ||=" => type(nothing) errors(NoSuchField@0,MissingRightOperand@[2-4]),

    assign_missing_left: "= 1" => type(error) errors(MissingLeftOperand@0),
    assign_plus_missing_left: "+= 1" => type(error) errors(MissingLeftOperand@[0-1]),
    assign_minus_missing_left: "-= 1" => type(error) errors(MissingLeftOperand@[0-1]),
    assign_multiply_missing_left: "*= 1" => type(error) errors(MissingLeftOperand@[0-1]),
    assign_divide_missing_left: "/= 1" => type(error) errors(MissingLeftOperand@[0-1]),
    assign_and_missing_left: "&&= false" => type(error) errors(MissingLeftOperand@[0-2]),
    assign_or_missing_left: "||= true" => type(error) errors(MissingLeftOperand@[0-2]),

    assign_missing_both: "=" => type(error) errors(MissingLeftOperand@0,MissingRightOperand@0),
    assign_plus_missing_both: "+=" => type(error) errors(MissingLeftOperand@[0-1],MissingRightOperand@[0-1]),
    assign_minus_missing_both: "-=" => type(error) errors(MissingLeftOperand@[0-1],MissingRightOperand@[0-1]),
    assign_multiply_missing_both: "*=" => type(error) errors(MissingLeftOperand@[0-1],MissingRightOperand@[0-1]),
    assign_divide_missing_both: "/=" => type(error) errors(MissingLeftOperand@[0-1],MissingRightOperand@[0-1]),
    assign_and_missing_both: "&&=" => type(error) errors(MissingLeftOperand@[0-2],MissingRightOperand@[0-2]),
    assign_or_missing_both: "||=" => type(error) errors(MissingLeftOperand@[0-2],MissingRightOperand@[0-2]),

    increment_no_operand: "++" => type(error) errors(UnrecognizedOperator@[0-1]),
    decrement_no_operand: "--" => type(error) errors(UnrecognizedOperator@[0-1]),

    //
    // Test assignment to non-properties
    //
    assign_non_field:         "1 = 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    assign_plus_non_field:    "1 += 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    assign_minus_non_field:   "1 -= 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    assign_times_non_field:   "1 *= 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    assign_divide_non_field:  "1 /= 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    increment_post_non_field: "1++" => type(error) error(LeftSideOfIncrementOrDecrementMustBeIdentifier@0),
    decrement_post_non_field: "1--" => type(error) error(LeftSideOfIncrementOrDecrementMustBeIdentifier@0),
    increment_pre_non_field:  "++1" => type(error) error(RightSideOfIncrementOrDecrementMustBeIdentifier@2),
    decrement_pre_non_field:  "--1" => type(error) error(RightSideOfIncrementOrDecrementMustBeIdentifier@2),

    // TODO make these work--right now we only print one token, need to print whole expr
    // assign_non_field_expr:         "1+2 = 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    // assign_plus_non_field_expr:    "1+2 += 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    // assign_minus_non_field_expr:   "1+2 -= 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    // assign_times_non_field_expr:   "1+2 *= 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    // assign_divide_non_field_expr:  "1+2 /= 1" => type(error) error(LeftSideOfAssignmentMustBeIdentifier@0),
    // increment_post_non_field_expr: "(1+2)++" => type(error) error(LeftSideOfIncrementOrDecrementMustBeIdentifier@0),
    // decrement_post_non_field_expr: "(1+2)--" => type(error) error(LeftSideOfIncrementOrDecrementMustBeIdentifier@0),
    // increment_pre_non_field_expr:  "++(1+2)" => type(error) error(RightSideOfIncrementOrDecrementMustBeIdentifier@2),
    // decrement_pre_non_field_expr:  "--(1+2)" => type(error) error(RightSideOfIncrementOrDecrementMustBeIdentifier@2),

    //
    // Test that errors during the actual statement are propagated
    //

    assign_error:        ":a = 1 + true" => type(nothing) errors(BadTypeRightOperand@7),
    reassign_error:      ":a = 1; a  = 1 + true"    => type(nothing) errors(BadTypeRightOperand@15),
    assign_plus_error:   ":a = 1; a += 1 + true"    => type(nothing) errors(BadTypeRightOperand@15),
    assign_minus_error:  ":a = 1; a -= 1 + true"    => type(nothing) errors(BadTypeRightOperand@15),
    assign_times_error:  ":a = 1; a *= 1 + true"    => type(nothing) errors(BadTypeRightOperand@15),
    assign_divide_error: ":a = 1; a /= 1 + true"    => type(nothing) errors(BadTypeRightOperand@15),
    assign_and_error:    ":a = true; a &&= true && 1"  => type(nothing) errors(BadTypeRightOperand@[22-23]),
    assign_or_error:     ":a = true; a ||= false && 1" => type(nothing) errors(BadTypeRightOperand@[23-24]),

    assign_error_ref:        ":a = 1 + true; a" => type(error) errors(BadTypeRightOperand@7),
    reassign_error_ref:      ":a = 1; a  = 1 + true; a"    => type(error) errors(BadTypeRightOperand@15),
    assign_plus_error_ref:   ":a = 1; a += 1 + true; a"    => type(error) errors(BadTypeRightOperand@15),
    assign_minus_error_ref:  ":a = 1; a -= 1 + true; a"    => type(error) errors(BadTypeRightOperand@15),
    assign_times_error_ref:  ":a = 1; a *= 1 + true; a"    => type(error) errors(BadTypeRightOperand@15),
    assign_divide_error_ref: ":a = 1; a /= 1 + true; a"    => type(error) errors(BadTypeRightOperand@15),
    assign_and_error_ref:    ":a = true; a &&= true && 1; a"  => type(error) errors(BadTypeRightOperand@[22-23]),
    assign_or_error_ref:     ":a = true; a ||= false && 1; a" => type(error) errors(BadTypeRightOperand@[23-24]),

    assign_error_ref_twice: ":a = 1 + true; a + a" => type(error) errors(BadTypeRightOperand@7),

    //
    // Test assignment to undefined values (ones that haven't been set)
    //

    assign_undeclared:        "a = 1"       => type(nothing),
    assign_plus_undeclared:   "a += 1"      => type(nothing) errors(NoSuchField@0),
    assign_minus_undeclared:  "a -= 1"      => type(nothing) errors(NoSuchField@0),
    assign_times_undeclared:  "a *= 1"      => type(nothing) errors(NoSuchField@0),
    assign_divide_undeclared: "a /= 1"      => type(nothing) errors(NoSuchField@0),
    assign_and_undeclared:    "a &&= true"  => type(nothing) errors(NoSuchField@0),
    assign_or_undeclared:     "a ||= false" => type(nothing) errors(NoSuchField@0),
    increment_pre_undeclared: "++a"  => type(nothing) errors(NoSuchField@2),
    decrement_pre_undeclared: "--a"  => type(nothing) errors(NoSuchField@2),
    increment_post_undeclared: "a++" => type(nothing) errors(NoSuchField@0),
    decrement_post_undeclared: "a--" => type(nothing) errors(NoSuchField@0),

    assign_plus_undeclared_bad_type:   "a += true" => type(nothing) errors(NoSuchField@0,BadTypeRightOperand@[2-3]),
    assign_minus_undeclared_bad_type:  "a -= true" => type(nothing) errors(NoSuchField@0,BadTypeRightOperand@[2-3]),
    assign_times_undeclared_bad_type:  "a *= true" => type(nothing) errors(NoSuchField@0,BadTypeRightOperand@[2-3]),
    assign_divide_undeclared_bad_type: "a /= true" => type(nothing) errors(NoSuchField@0,BadTypeRightOperand@[2-3]),
    assign_and_undeclared_bad_type:    "a &&= 1"   => type(nothing) errors(NoSuchField@0,BadTypeRightOperand@[2-4]),
    assign_or_undeclared_bad_type:     "a ||= 2"   => type(nothing) errors(NoSuchField@0,BadTypeRightOperand@[2-4]),

    //
    // Test assignment to undefined values (ones that haven't been set)
    //

    assign_plus_undefined:   ":a; a += 1"      => type(nothing) errors(FieldNotSet@1),
    assign_minus_undefined:  ":a; a -= 1"      => type(nothing) errors(FieldNotSet@1),
    assign_times_undefined:  ":a; a *= 1"      => type(nothing) errors(FieldNotSet@1),
    assign_divide_undefined: ":a; a /= 1"      => type(nothing) errors(FieldNotSet@1),
    assign_and_undefined:    ":a; a &&= true"  => type(nothing) errors(FieldNotSet@1),
    assign_or_undefined:     ":a; a ||= false" => type(nothing) errors(FieldNotSet@1),
    increment_pre_undefined:  ":a; ++a"  => type(nothing) errors(FieldNotSet@1),
    decrement_pre_undefined:  ":a; --a"  => type(nothing) errors(FieldNotSet@1),
    increment_post_undefined: ":a; a++" => type(nothing) errors(FieldNotSet@1),
    decrement_post_undefined: ":a; a--" => type(nothing) errors(FieldNotSet@1),

    assign_plus_undefined_bad_type:   ":a; a += true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_minus_undefined_bad_type:  ":a; a -= true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_times_undefined_bad_type:  ":a; a *= true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_divide_undefined_bad_type: ":a; a /= true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_and_undefined_bad_type:    ":a; a &&= 1"   => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[6-8]),
    assign_or_undefined_bad_type:     ":a; a ||= 2"   => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[6-8]),

    //
    // Test behavior of assignment operations with declarations on the LHS
    //

    assign_plus_declaration:   ":a += 1"      => type(nothing) errors(FieldNotSet@1),
    assign_minus_declaration:  ":a -= 1"      => type(nothing) errors(FieldNotSet@1),
    assign_times_declaration:  ":a *= 1"      => type(nothing) errors(FieldNotSet@1),
    assign_divide_declaration: ":a /= 1"      => type(nothing) errors(FieldNotSet@1),
    assign_and_declaration:    ":a &&= true"  => type(nothing) errors(FieldNotSet@1),
    assign_or_declaration:     ":a ||= false" => type(nothing) errors(FieldNotSet@1),
    increment_pre_declaration: "++:a"  => type(nothing) errors(FieldNotSet@3),
    decrement_pre_declaration: "--:a"  => type(nothing) errors(FieldNotSet@3),
    increment_post_declaration: ":a++" => type(nothing) errors(FieldNotSet@1),
    decrement_post_declaration: ":a--" => type(nothing) errors(FieldNotSet@1),

    assign_plus_declaration_bad_type:   ":a += true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[3-4]),
    assign_minus_declaration_bad_type:  ":a -= true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[3-4]),
    assign_times_declaration_bad_type:  ":a *= true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[3-4]),
    assign_divide_declaration_bad_type: ":a /= true" => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[3-4]),
    assign_and_declaration_bad_type:    ":a &&= 1"   => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[3-5]),
    assign_or_declaration_bad_type:     ":a ||= 2"   => type(nothing) errors(FieldNotSet@1,BadTypeRightOperand@[3-5]),

    //
    // Test behavior of undefined self references
    //

    assign_self_ref:          ":a = a + 1" => type(nothing) errors(NoSuchField@5),
    reassign_self_ref:        "a = a + 1" => type(nothing) errors(NoSuchField@4),
    assign_plus_self_ref:     "a += a" => type(nothing) errors(NoSuchField@0),
    assign_minus_self_ref:    "a -= a" => type(nothing) errors(NoSuchField@0),
    assign_multiply_self_ref: "a *= a" => type(nothing) errors(NoSuchField@0),
    assign_divide_self_ref:   "a /= a" => type(nothing) errors(NoSuchField@0),
    assign_and_self_ref:      "a &&= a" => type(nothing) errors(NoSuchField@0),
    assign_or_self_ref:       "a ||= a" => type(nothing) errors(NoSuchField@0),

    assign_plus_declaration_self_ref:     ":a += a" => type(nothing) errors(FieldNotSet@1),
    assign_minus_declaration_self_ref:    ":a -= a" => type(nothing) errors(FieldNotSet@1),
    assign_multiply_declaration_self_ref: ":a *= a" => type(nothing) errors(FieldNotSet@1),
    assign_divide_declaration_self_ref:   ":a /= a" => type(nothing) errors(FieldNotSet@1),
    assign_and_declaration_self_ref:      ":a &&= a" => type(nothing) errors(FieldNotSet@1),
    assign_or_declaration_self_ref:       ":a ||= a" => type(nothing) errors(FieldNotSet@1),

    //
    // Test bad type errors on assignment
    //

    assign_plus_bad_type_left:      ":a = true; a += 2"    => type(nothing) errors(BadTypeLeftOperand@[13-14]),
    assign_plus_bad_type_right:     ":a = 2;    a += true" => type(nothing) errors(BadTypeRightOperand@[13-14]),
    assign_plus_bad_type_both:      ":a = true; a += true" => type(nothing) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),
    assign_minus_bad_type_left:     ":a = true; a -= 2"    => type(nothing) errors(BadTypeLeftOperand@[13-14]),
    assign_minus_bad_type_right:    ":a = 2;    a -= true" => type(nothing) errors(BadTypeRightOperand@[13-14]),
    assign_minus_bad_type_both:     ":a = true; a -= true" => type(nothing) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),
    assign_multiply_bad_type_left:  ":a = true; a *= 2"    => type(nothing) errors(BadTypeLeftOperand@[13-14]),
    assign_multiply_bad_type_right: ":a = 2;    a *= true" => type(nothing) errors(BadTypeRightOperand@[13-14]),
    assign_multiply_bad_type_both:  ":a = true; a *= true" => type(nothing) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),
    assign_divide_bad_type_left:    ":a = true; a /= 2"    => type(nothing) errors(BadTypeLeftOperand@[13-14]),
    assign_divide_bad_type_right:   ":a = 2;    a /= true" => type(nothing) errors(BadTypeRightOperand@[13-14]),
    assign_divide_bad_type_both:    ":a = true; a /= true" => type(nothing) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),

    assign_and_bad_type_left:       ":a = 2;    a &&= true" => type(nothing) errors(BadTypeLeftOperand@[13-15]),
    assign_and_bad_type_right:      ":a = true; a &&= 2"    => type(nothing) errors(BadTypeRightOperand@[13-15]),
    assign_and_bad_type_both:       ":a = 2;    a &&= 2"    => type(nothing) errors(BadTypeLeftOperand@[13-15],BadTypeRightOperand@[13-15]),
    assign_or_bad_type_left:        ":a = 2;    a &&= true" => type(nothing) errors(BadTypeLeftOperand@[13-15]),
    assign_or_bad_type_right:       ":a = true; a &&= 2"    => type(nothing) errors(BadTypeRightOperand@[13-15]),
    assign_or_bad_type_both:        ":a = 2;    a &&= 2"    => type(nothing) errors(BadTypeLeftOperand@[13-15],BadTypeRightOperand@[13-15]),

    // Test error reporting (and result values) for references to failed assignments

    declaration_declaration_ref:   ":a; a"           => type(error) errors(FieldNotSet@1),
    assign_plus_declaration_ref:   ":a += 1; a"      => type(error) errors(FieldNotSet@1),
    assign_minus_declaration_ref:  ":a -= 1; a"      => type(error) errors(FieldNotSet@1),
    assign_times_declaration_ref:  ":a *= 1; a"      => type(error) errors(FieldNotSet@1),
    assign_divide_declaration_ref: ":a /= 1; a"      => type(error) errors(FieldNotSet@1),
    assign_and_declaration_ref:    ":a &&= true; a"  => type(error) errors(FieldNotSet@1),
    assign_or_declaration_ref:     ":a ||= false; a" => type(error) errors(FieldNotSet@1),

    assign_plus_declaration_bad_type_ref:   ":a; a += true; a" => type(error) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_minus_declaration_bad_type_ref:  ":a; a -= true; a" => type(error) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_times_declaration_bad_type_ref:  ":a; a *= true; a" => type(error) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_divide_declaration_bad_type_ref: ":a; a /= true; a" => type(error) errors(FieldNotSet@1,BadTypeRightOperand@[6-7]),
    assign_and_declaration_bad_type_ref:    ":a; a &&= 1; a"   => type(error) errors(FieldNotSet@1,BadTypeRightOperand@[6-8]),
    assign_or_declaration_bad_type_ref:     ":a; a ||= 2; a"   => type(error) errors(FieldNotSet@1,BadTypeRightOperand@[6-8]),

    assign_plus_bad_type_left_ref:      ":a = true; a += 2; a"    => type(error) errors(BadTypeLeftOperand@[13-14]),
    assign_plus_bad_type_right_ref:     ":a = 2;    a += true; a" => type(error) errors(BadTypeRightOperand@[13-14]),
    assign_plus_bad_type_both_ref:      ":a = true; a += true; a" => type(error) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),
    assign_minus_bad_type_left_ref:     ":a = true; a -= 2; a"    => type(error) errors(BadTypeLeftOperand@[13-14]),
    assign_minus_bad_type_right_ref:    ":a = 2;    a -= true; a" => type(error) errors(BadTypeRightOperand@[13-14]),
    assign_minus_bad_type_both_ref:     ":a = true; a -= true; a" => type(error) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),
    assign_multiply_bad_type_left_ref:  ":a = true; a *= 2; a"    => type(error) errors(BadTypeLeftOperand@[13-14]),
    assign_multiply_bad_type_right_ref: ":a = 2;    a *= true; a" => type(error) errors(BadTypeRightOperand@[13-14]),
    assign_multiply_bad_type_both_ref:  ":a = true; a *= true; a" => type(error) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),
    assign_divide_bad_type_left_ref:    ":a = true; a /= 2; a"    => type(error) errors(BadTypeLeftOperand@[13-14]),
    assign_divide_bad_type_right_ref:   ":a = 2;    a /= true; a" => type(error) errors(BadTypeRightOperand@[13-14]),
    assign_divide_bad_type_both_ref:    ":a = true; a /= true; a" => type(error) errors(BadTypeLeftOperand@[13-14],BadTypeRightOperand@[13-14]),

    assign_and_bad_type_left_ref:       ":a = 2;    a &&= true; a" => type(error) errors(BadTypeLeftOperand@[13-15]),
    assign_and_bad_type_right_ref:      ":a = true; a &&= 2; a"    => type(error) errors(BadTypeRightOperand@[13-15]),
    assign_and_bad_type_both_ref:       ":a = 2;    a &&= 2; a"    => type(error) errors(BadTypeLeftOperand@[13-15],BadTypeRightOperand@[13-15]),
    assign_or_bad_type_left_ref:        ":a = 2;    a &&= true; a" => type(error) errors(BadTypeLeftOperand@[13-15]),
    assign_or_bad_type_right_ref:       ":a = true; a &&= 2; a"    => type(error) errors(BadTypeRightOperand@[13-15]),
    assign_or_bad_type_both_ref:        ":a = 2;    a &&= 2; a"    => type(error) errors(BadTypeLeftOperand@[13-15],BadTypeRightOperand@[13-15]),
}
