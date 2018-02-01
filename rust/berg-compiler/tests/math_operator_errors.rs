#![recursion_limit = "256"]
#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    addmul_missing_operator_precedence: "1 * + 3" => error(MissingOperand@2),
    muladd_missing_operator_precedence: "1 + * 3" => error(MissingOperand@4),
    addparen_missing_operator_precedence: "(1 + )" => error(MissingOperand@3),
    parenadd_missing_operator_precedence: "( + 1)" => error(MissingOperand@2),

    div0_0: "0/0" => error(DivideByZero@2),
    div1_0: "1/0" => error(DivideByZero@2),

    trailing_neg: "0-" => error(UnsupportedOperator@1),
    trailing_pos: "0+" => error(UnsupportedOperator@1),
    sub_only:      "-" => error(MissingOperand@0),
    add_only:      "+" => error(MissingOperand@0),
    plus_minus: "1+-2" => error(UnsupportedOperator@[1-2]),

    // + errors
    add_true_1: "true+1" => error(UnsupportedOperator@4),
    add_1_true: "1+true" => error(BadType@[2-5]),
    add_false_1: "false+1" => error(UnsupportedOperator@5),
    add_1_false: "1+false" => error(BadType@[2-6]),
    add_nothing_1: "()+1" => error(UnsupportedOperator@2),
    add_1_nothing: "1+()" => error(BadType@[2-3]),
    add_error_1: "1/0+1" => error(DivideByZero@2),
    add_1_error: "1+1/0" => error(DivideByZero@4),

    add_true_true: "true+true" => error(UnsupportedOperator@4),
    add_false_true: "false+true" => error(UnsupportedOperator@5),
    add_true_false: "true+false" => error(UnsupportedOperator@4),
    add_false_false: "false+false" => error(UnsupportedOperator@5),
    add_true_error: "true+1/0" => error(UnsupportedOperator@4),
    add_error_true: "1/0+true" => error(DivideByZero@2),
    add_false_error: "false+1/0" => error(UnsupportedOperator@5),
    add_error_false: "1/0+false" => error(DivideByZero@2),
    add_true_nothing: "true+()" => error(UnsupportedOperator@4),
    add_nothing_true: "()+true" => error(UnsupportedOperator@2),
    add_false_nothing: "false+()" => error(UnsupportedOperator@5),
    add_nothing_false: "()+false" => error(UnsupportedOperator@2),

    add_error_error: "1/0+1/0" => error(DivideByZero@2),
    add_error_nothing: "1/0+()" => error(DivideByZero@2),
    add_nothing_error: "()+1/0" => error(UnsupportedOperator@2),
    add_nothing_nothing: "()+()" => error(UnsupportedOperator@2),

    // - errors
    sub_true_1: "true-1" => error(UnsupportedOperator@4),
    sub_1_true: "1-true" => error(BadType@[2-5]),
    sub_false_1: "false-1" => error(UnsupportedOperator@5),
    sub_1_false: "1-false" => error(BadType@[2-6]),
    sub_nothing_1: "()-1" => error(UnsupportedOperator@2),
    sub_1_nothing: "1-()" => error(BadType@[2-3]),
    sub_error_1: "1/0-1" => error(DivideByZero@2),
    sub_1_error: "1-1/0" => error(DivideByZero@4),

    sub_true_true: "true-true" => error(UnsupportedOperator@4),
    sub_false_true: "false-true" => error(UnsupportedOperator@5),
    sub_true_false: "true-false" => error(UnsupportedOperator@4),
    sub_false_false: "false-false" => error(UnsupportedOperator@5),
    sub_true_error: "true-1/0" => error(UnsupportedOperator@4),
    sub_error_true: "1/0-true" => error(DivideByZero@2),
    sub_false_error: "false-1/0" => error(UnsupportedOperator@5),
    sub_error_false: "1/0-false" => error(DivideByZero@2),
    sub_true_nothing: "true-()" => error(UnsupportedOperator@4),
    sub_nothing_true: "()-true" => error(UnsupportedOperator@2),
    sub_false_nothing: "false-()" => error(UnsupportedOperator@5),
    sub_nothing_false: "()-false" => error(UnsupportedOperator@2),

    sub_error_error: "1/0-1/0" => error(DivideByZero@2),
    sub_error_nothing: "1/0-()" => error(DivideByZero@2),
    sub_nothing_error: "()-1/0" => error(UnsupportedOperator@2),
    sub_nothing_nothing: "()-()" => error(UnsupportedOperator@2),


    // * errors
    mul_true_1: "true*1" => error(UnsupportedOperator@4),
    mul_1_true: "1*true" => error(BadType@[2-5]),
    mul_false_1: "false*1" => error(UnsupportedOperator@5),
    mul_1_false: "1*false" => error(BadType@[2-6]),
    mul_nothing_1: "()*1" => error(UnsupportedOperator@2),
    mul_1_nothing: "1*()" => error(BadType@[2-3]),
    mul_error_1: "(1/0)*1" => error(DivideByZero@3),
    mul_1_error: "1*(1/0)" => error(DivideByZero@5),

    mul_true_true: "true*true" => error(UnsupportedOperator@4),
    mul_false_true: "false*true" => error(UnsupportedOperator@5),
    mul_true_false: "true*false" => error(UnsupportedOperator@4),
    mul_false_false: "false*false" => error(UnsupportedOperator@5),
    mul_true_error: "true*(1/0)" => error(UnsupportedOperator@4),
    mul_error_true: "(1/0)*true" => error(DivideByZero@3),
    mul_false_error: "false*(1/0)" => error(UnsupportedOperator@5),
    mul_error_false: "(1/0)*false" => error(DivideByZero@3),
    mul_true_nothing: "true*()" => error(UnsupportedOperator@4),
    mul_nothing_true: "()*true" => error(UnsupportedOperator@2),
    mul_false_nothing: "false*()" => error(UnsupportedOperator@5),
    mul_nothing_false: "()*false" => error(UnsupportedOperator@2),

    mul_error_error: "(1/0)*(1/0)" => error(DivideByZero@3),
    mul_error_nothing: "(1/0)*()" => error(DivideByZero@3),
    mul_nothing_error: "()*(1/0)" => error(UnsupportedOperator@2),
    mul_nothing_nothing: "()*()" => error(UnsupportedOperator@2),

    // / errors
    div_true_1: "true/1" => error(UnsupportedOperator@4),
    div_1_true: "1/true" => error(BadType@[2-5]),
    div_false_1: "false/1" => error(UnsupportedOperator@5),
    div_1_false: "1/false" => error(BadType@[2-6]),
    div_nothing_1: "()/1" => error(UnsupportedOperator@2),
    div_1_nothing: "1/()" => error(BadType@[2-3]),
    div_error_1: "(1/0)/1" => error(DivideByZero@3),
    div_1_error: "1/(1/0)" => error(DivideByZero@5),

    div_true_true: "true/true" => error(UnsupportedOperator@4),
    div_false_true: "false/true" => error(UnsupportedOperator@5),
    div_true_false: "true/false" => error(UnsupportedOperator@4),
    div_false_false: "false/false" => error(UnsupportedOperator@5),
    div_true_error: "true/(1/0)" => error(UnsupportedOperator@4),
    div_error_true: "(1/0)/true" => error(DivideByZero@3),
    div_false_error: "false/(1/0)" => error(UnsupportedOperator@5),
    div_error_false: "(1/0)/false" => error(DivideByZero@3),
    div_true_nothing: "true/()" => error(UnsupportedOperator@4),
    div_nothing_true: "()/true" => error(UnsupportedOperator@2),
    div_false_nothing: "false/()" => error(UnsupportedOperator@5),
    div_nothing_false: "()/false" => error(UnsupportedOperator@2),

    div_error_error: "(1/0)/(1/0)" => error(DivideByZero@3),
    div_error_nothing: "(1/0)/()" => error(DivideByZero@3),
    div_nothing_error: "()/(1/0)" => error(UnsupportedOperator@2),
    div_nothing_nothing: "()/()" => error(UnsupportedOperator@2),

    // Negative - errors
    neg_true: "-true" => error(UnsupportedOperator@0),
    neg_false: "-false" => error(UnsupportedOperator@0),
    neg_nothing: "-()" => error(UnsupportedOperator@0),
    neg_error: "-(1/0)" => error(DivideByZero@4),

    // Positive + errors
    pos_true: "+true" => error(UnsupportedOperator@0),
    pos_false: "+false" => error(UnsupportedOperator@0),
    pos_nothing: "+()" => error(UnsupportedOperator@0),
    pos_error: "+(1/0)" => error(DivideByZero@4),
}
