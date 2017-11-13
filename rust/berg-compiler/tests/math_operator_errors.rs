#[macro_use]
pub mod compiler_test;

compiler_tests! {
    addmul_missing_operator_precedence: "1 * + 3" => errors(MissingRightOperand@2) type(error),
    muladd_missing_operator_precedence: "1 + * 3" => errors(MissingLeftOperand@4) type(error),
    addparen_missing_operator_precedence: "(1 + )" => errors(MissingRightOperand@3) type(error),
    parenadd_missing_operator_precedence: "( + 1)" => errors(MissingLeftOperand@2) type(error),

    div0_0: "0/0" => error(DivideByZero@1) type(error),
    div1_0: "1/0" => error(DivideByZero@1) type(error),

    trailing_neg: "0-" => error(UnrecognizedOperator@1) type(error),
    trailing_pos: "0+" => error(UnrecognizedOperator@1) type(error),
    sub_only:      "-" => errors(MissingLeftOperand@0,MissingRightOperand@0) type(error),
    add_only:      "+" => errors(MissingLeftOperand@0,MissingRightOperand@0) type(error),
    plus_minus: "1+-2" => error(UnrecognizedOperator@[1-2]) type(error),

    // + errors
    add_true_1: "true+1" => errors(BadTypeLeftOperand@4) type(error),
    add_1_true: "1+true" => errors(BadTypeRightOperand@1) type(error),
    add_false_1: "false+1" => errors(BadTypeLeftOperand@5) type(error),
    add_1_false: "1+false" => errors(BadTypeRightOperand@1) type(error),
    add_nothing_1: "()+1" => errors(BadTypeLeftOperand@2) type(error),
    add_1_nothing: "1+()" => errors(BadTypeRightOperand@1) type(error),
    add_error_1: "1/0+1" => errors(DivideByZero@1) type(error),
    add_1_error: "1+1/0" => errors(DivideByZero@3) type(error),

    add_true_true: "true+true" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    add_false_true: "false+true" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    add_true_false: "true+false" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    add_false_false: "false+false" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    add_true_error: "true+1/0" => errors(BadTypeLeftOperand@4,DivideByZero@6) type(error),
    add_error_true: "1/0+true" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    add_false_error: "false+1/0" => errors(BadTypeLeftOperand@5,DivideByZero@7) type(error),
    add_error_false: "1/0+false" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    add_true_nothing: "true+()" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    add_nothing_true: "()+true" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    add_false_nothing: "false+()" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    add_nothing_false: "()+false" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    add_error_error: "1/0+1/0" => errors(DivideByZero@1,DivideByZero@5) type(error),
    add_error_nothing: "1/0+()" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    add_nothing_error: "()+1/0" => errors(BadTypeLeftOperand@2,DivideByZero@4) type(error),
    add_nothing_nothing: "()+()" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    // - errors
    sub_true_1: "true-1" => errors(BadTypeLeftOperand@4) type(error),
    sub_1_true: "1-true" => errors(BadTypeRightOperand@1) type(error),
    sub_false_1: "false-1" => errors(BadTypeLeftOperand@5) type(error),
    sub_1_false: "1-false" => errors(BadTypeRightOperand@1) type(error),
    sub_nothing_1: "()-1" => errors(BadTypeLeftOperand@2) type(error),
    sub_1_nothing: "1-()" => errors(BadTypeRightOperand@1) type(error),
    sub_error_1: "1/0-1" => errors(DivideByZero@1) type(error),
    sub_1_error: "1-1/0" => errors(DivideByZero@3) type(error),

    sub_true_true: "true-true" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    sub_false_true: "false-true" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    sub_true_false: "true-false" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    sub_false_false: "false-false" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    sub_true_error: "true-1/0" => errors(BadTypeLeftOperand@4,DivideByZero@6) type(error),
    sub_error_true: "1/0-true" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    sub_false_error: "false-1/0" => errors(BadTypeLeftOperand@5,DivideByZero@7) type(error),
    sub_error_false: "1/0-false" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    sub_true_nothing: "true-()" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    sub_nothing_true: "()-true" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    sub_false_nothing: "false-()" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    sub_nothing_false: "()-false" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    sub_error_error: "1/0-1/0" => errors(DivideByZero@1,DivideByZero@5) type(error),
    sub_error_nothing: "1/0-()" => errors(DivideByZero@1,BadTypeRightOperand@3) type(error),
    sub_nothing_error: "()-1/0" => errors(BadTypeLeftOperand@2,DivideByZero@4) type(error),
    sub_nothing_nothing: "()-()" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),


    // * errors
    mul_true_1: "true*1" => errors(BadTypeLeftOperand@4) type(error),
    mul_1_true: "1*true" => errors(BadTypeRightOperand@1) type(error),
    mul_false_1: "false*1" => errors(BadTypeLeftOperand@5) type(error),
    mul_1_false: "1*false" => errors(BadTypeRightOperand@1) type(error),
    mul_nothing_1: "()*1" => errors(BadTypeLeftOperand@2) type(error),
    mul_1_nothing: "1*()" => errors(BadTypeRightOperand@1) type(error),
    mul_error_1: "(1/0)*1" => errors(DivideByZero@2) type(error),
    mul_1_error: "1*(1/0)" => errors(DivideByZero@4) type(error),

    mul_true_true: "true*true" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    mul_false_true: "false*true" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    mul_true_false: "true*false" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    mul_false_false: "false*false" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    mul_true_error: "true*(1/0)" => errors(BadTypeLeftOperand@4,DivideByZero@7) type(error),
    mul_error_true: "(1/0)*true" => errors(DivideByZero@2,BadTypeRightOperand@5) type(error),
    mul_false_error: "false*(1/0)" => errors(BadTypeLeftOperand@5,DivideByZero@8) type(error),
    mul_error_false: "(1/0)*false" => errors(DivideByZero@2,BadTypeRightOperand@5) type(error),
    mul_true_nothing: "true*()" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    mul_nothing_true: "()*true" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    mul_false_nothing: "false*()" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    mul_nothing_false: "()*false" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    mul_error_error: "(1/0)*(1/0)" => errors(DivideByZero@2,DivideByZero@8) type(error),
    mul_error_nothing: "(1/0)*()" => errors(DivideByZero@2,BadTypeRightOperand@5) type(error),
    mul_nothing_error: "()*(1/0)" => errors(BadTypeLeftOperand@2,DivideByZero@5) type(error),
    mul_nothing_nothing: "()*()" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    // / errors
    div_true_1: "true/1" => errors(BadTypeLeftOperand@4) type(error),
    div_1_true: "1/true" => errors(BadTypeRightOperand@1) type(error),
    div_false_1: "false/1" => errors(BadTypeLeftOperand@5) type(error),
    div_1_false: "1/false" => errors(BadTypeRightOperand@1) type(error),
    div_nothing_1: "()/1" => errors(BadTypeLeftOperand@2) type(error),
    div_1_nothing: "1/()" => errors(BadTypeRightOperand@1) type(error),
    div_error_1: "(1/0)/1" => errors(DivideByZero@2) type(error),
    div_1_error: "1/(1/0)" => errors(DivideByZero@4) type(error),

    div_true_true: "true/true" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    div_false_true: "false/true" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    div_true_false: "true/false" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    div_false_false: "false/false" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    div_true_error: "true/(1/0)" => errors(BadTypeLeftOperand@4,DivideByZero@7) type(error),
    div_error_true: "(1/0)/true" => errors(DivideByZero@2,BadTypeRightOperand@5) type(error),
    div_false_error: "false/(1/0)" => errors(BadTypeLeftOperand@5,DivideByZero@8) type(error),
    div_error_false: "(1/0)/false" => errors(DivideByZero@2,BadTypeRightOperand@5) type(error),
    div_true_nothing: "true/()" => errors(BadTypeLeftOperand@4,BadTypeRightOperand@4) type(error),
    div_nothing_true: "()/true" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),
    div_false_nothing: "false/()" => errors(BadTypeLeftOperand@5,BadTypeRightOperand@5) type(error),
    div_nothing_false: "()/false" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    div_error_error: "(1/0)/(1/0)" => errors(DivideByZero@2,DivideByZero@8) type(error),
    div_error_nothing: "(1/0)/()" => errors(DivideByZero@2,BadTypeRightOperand@5) type(error),
    div_nothing_error: "()/(1/0)" => errors(BadTypeLeftOperand@2,DivideByZero@5) type(error),
    div_nothing_nothing: "()/()" => errors(BadTypeLeftOperand@2,BadTypeRightOperand@2) type(error),

    // Negative - errors
    neg_true: "-true" => errors(BadTypeRightOperand@0) type(error),
    neg_false: "-false" => errors(BadTypeRightOperand@0) type(error),
    neg_nothing: "-()" => errors(BadTypeRightOperand@0) type(error),
    neg_error: "-(1/0)" => errors(DivideByZero@3) type(error),

    // Positive + errors
    pos_true: "+true" => errors(BadTypeRightOperand@0) type(error),
    pos_false: "+false" => errors(BadTypeRightOperand@0) type(error),
    pos_nothing: "+()" => errors(BadTypeRightOperand@0) type(error),
    pos_error: "+(1/0)" => errors(DivideByZero@3) type(error),
}
