#[macro_use]
pub mod compiler_test;

compiler_tests! {
    addmul_missing_operator_precedence: "1 * + 3" => errors(MissingOperand@2),
    muladd_missing_operator_precedence: "1 + * 3" => errors(MissingOperand@4),
    addparen_missing_operator_precedence: "(1 + )" => errors(MissingOperand@3),
    parenadd_missing_operator_precedence: "( + 1)" => errors(MissingOperand@2),

    div0_0: "0/0" => error(DivideByZero@1),
    div1_0: "1/0" => error(DivideByZero@1),

    trailing_neg: "0-" => error(UnrecognizedOperator@1),
    trailing_pos: "0+" => error(UnrecognizedOperator@1),
    sub_only:      "-" => errors(MissingOperand@0,MissingOperand@0),
    add_only:      "+" => errors(MissingOperand@0,MissingOperand@0),
    plus_minus: "1+-2" => error(UnrecognizedOperator@[1-2]),

    // + errors
    add_true_1: "true+1" => errors(BadType@4),
    add_1_true: "1+true" => errors(BadType@1),
    add_false_1: "false+1" => errors(BadType@5),
    add_1_false: "1+false" => errors(BadType@1),
    add_nothing_1: "()+1" => errors(BadType@2),
    add_1_nothing: "1+()" => errors(BadType@1),
    add_error_1: "1/0+1" => errors(DivideByZero@1),
    add_1_error: "1+1/0" => errors(DivideByZero@3),

    add_true_true: "true+true" => errors(BadType@4,BadType@4),
    add_false_true: "false+true" => errors(BadType@5,BadType@5),
    add_true_false: "true+false" => errors(BadType@4,BadType@4),
    add_false_false: "false+false" => errors(BadType@5,BadType@5),
    add_true_error: "true+1/0" => errors(BadType@4,DivideByZero@6),
    add_error_true: "1/0+true" => errors(DivideByZero@1,BadType@3),
    add_false_error: "false+1/0" => errors(BadType@5,DivideByZero@7),
    add_error_false: "1/0+false" => errors(DivideByZero@1,BadType@3),
    add_true_nothing: "true+()" => errors(BadType@4,BadType@4),
    add_nothing_true: "()+true" => errors(BadType@2,BadType@2),
    add_false_nothing: "false+()" => errors(BadType@5,BadType@5),
    add_nothing_false: "()+false" => errors(BadType@2,BadType@2),

    add_error_error: "1/0+1/0" => errors(DivideByZero@1,DivideByZero@5),
    add_error_nothing: "1/0+()" => errors(DivideByZero@1,BadType@3),
    add_nothing_error: "()+1/0" => errors(BadType@2,DivideByZero@4),
    add_nothing_nothing: "()+()" => errors(BadType@2,BadType@2),

    // - errors
    sub_true_1: "true-1" => errors(BadType@4),
    sub_1_true: "1-true" => errors(BadType@1),
    sub_false_1: "false-1" => errors(BadType@5),
    sub_1_false: "1-false" => errors(BadType@1),
    sub_nothing_1: "()-1" => errors(BadType@2),
    sub_1_nothing: "1-()" => errors(BadType@1),
    sub_error_1: "1/0-1" => errors(DivideByZero@1),
    sub_1_error: "1-1/0" => errors(DivideByZero@3),

    sub_true_true: "true-true" => errors(BadType@4,BadType@4),
    sub_false_true: "false-true" => errors(BadType@5,BadType@5),
    sub_true_false: "true-false" => errors(BadType@4,BadType@4),
    sub_false_false: "false-false" => errors(BadType@5,BadType@5),
    sub_true_error: "true-1/0" => errors(BadType@4,DivideByZero@6),
    sub_error_true: "1/0-true" => errors(DivideByZero@1,BadType@3),
    sub_false_error: "false-1/0" => errors(BadType@5,DivideByZero@7),
    sub_error_false: "1/0-false" => errors(DivideByZero@1,BadType@3),
    sub_true_nothing: "true-()" => errors(BadType@4,BadType@4),
    sub_nothing_true: "()-true" => errors(BadType@2,BadType@2),
    sub_false_nothing: "false-()" => errors(BadType@5,BadType@5),
    sub_nothing_false: "()-false" => errors(BadType@2,BadType@2),

    sub_error_error: "1/0-1/0" => errors(DivideByZero@1,DivideByZero@5),
    sub_error_nothing: "1/0-()" => errors(DivideByZero@1,BadType@3),
    sub_nothing_error: "()-1/0" => errors(BadType@2,DivideByZero@4),
    sub_nothing_nothing: "()-()" => errors(BadType@2,BadType@2),


    // * errors
    mul_true_1: "true*1" => errors(BadType@4),
    mul_1_true: "1*true" => errors(BadType@1),
    mul_false_1: "false*1" => errors(BadType@5),
    mul_1_false: "1*false" => errors(BadType@1),
    mul_nothing_1: "()*1" => errors(BadType@2),
    mul_1_nothing: "1*()" => errors(BadType@1),
    mul_error_1: "(1/0)*1" => errors(DivideByZero@2),
    mul_1_error: "1*(1/0)" => errors(DivideByZero@4),

    mul_true_true: "true*true" => errors(BadType@4,BadType@4),
    mul_false_true: "false*true" => errors(BadType@5,BadType@5),
    mul_true_false: "true*false" => errors(BadType@4,BadType@4),
    mul_false_false: "false*false" => errors(BadType@5,BadType@5),
    mul_true_error: "true*(1/0)" => errors(BadType@4,DivideByZero@7),
    mul_error_true: "(1/0)*true" => errors(DivideByZero@2,BadType@5),
    mul_false_error: "false*(1/0)" => errors(BadType@5,DivideByZero@8),
    mul_error_false: "(1/0)*false" => errors(DivideByZero@2,BadType@5),
    mul_true_nothing: "true*()" => errors(BadType@4,BadType@4),
    mul_nothing_true: "()*true" => errors(BadType@2,BadType@2),
    mul_false_nothing: "false*()" => errors(BadType@5,BadType@5),
    mul_nothing_false: "()*false" => errors(BadType@2,BadType@2),

    mul_error_error: "(1/0)*(1/0)" => errors(DivideByZero@2,DivideByZero@8),
    mul_error_nothing: "(1/0)*()" => errors(DivideByZero@2,BadType@5),
    mul_nothing_error: "()*(1/0)" => errors(BadType@2,DivideByZero@5),
    mul_nothing_nothing: "()*()" => errors(BadType@2,BadType@2),

    // / errors
    div_true_1: "true/1" => errors(BadType@4),
    div_1_true: "1/true" => errors(BadType@1),
    div_false_1: "false/1" => errors(BadType@5),
    div_1_false: "1/false" => errors(BadType@1),
    div_nothing_1: "()/1" => errors(BadType@2),
    div_1_nothing: "1/()" => errors(BadType@1),
    div_error_1: "(1/0)/1" => errors(DivideByZero@2),
    div_1_error: "1/(1/0)" => errors(DivideByZero@4),

    div_true_true: "true/true" => errors(BadType@4,BadType@4),
    div_false_true: "false/true" => errors(BadType@5,BadType@5),
    div_true_false: "true/false" => errors(BadType@4,BadType@4),
    div_false_false: "false/false" => errors(BadType@5,BadType@5),
    div_true_error: "true/(1/0)" => errors(BadType@4,DivideByZero@7),
    div_error_true: "(1/0)/true" => errors(DivideByZero@2,BadType@5),
    div_false_error: "false/(1/0)" => errors(BadType@5,DivideByZero@8),
    div_error_false: "(1/0)/false" => errors(DivideByZero@2,BadType@5),
    div_true_nothing: "true/()" => errors(BadType@4,BadType@4),
    div_nothing_true: "()/true" => errors(BadType@2,BadType@2),
    div_false_nothing: "false/()" => errors(BadType@5,BadType@5),
    div_nothing_false: "()/false" => errors(BadType@2,BadType@2),

    div_error_error: "(1/0)/(1/0)" => errors(DivideByZero@2,DivideByZero@8),
    div_error_nothing: "(1/0)/()" => errors(DivideByZero@2,BadType@5),
    div_nothing_error: "()/(1/0)" => errors(BadType@2,DivideByZero@5),
    div_nothing_nothing: "()/()" => errors(BadType@2,BadType@2),

    // Negative - errors
    neg_true: "-true" => errors(BadType@0),
    neg_false: "-false" => errors(BadType@0),
    neg_nothing: "-()" => errors(BadType@0),
    neg_error: "-(1/0)" => errors(DivideByZero@3),

    // Positive + errors
    pos_true: "+true" => errors(BadType@0),
    pos_false: "+false" => errors(BadType@0),
    pos_nothing: "+()" => errors(BadType@0),
    pos_error: "+(1/0)" => errors(DivideByZero@3),
}
