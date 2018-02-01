#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    right_semicolon: "1;" => value(Nothing),
    semicolon_right_space: "1; 2" => value(2),
    semicolon_both_space: "1 ; 2" => value(2),
    semicolon_sequence: "1;2" => value(2),
    semicolon_sequence_add: "1+1+1;2+2+2" => value(6),
    semicolon_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false;false||true&&4>=3+2*1" => value(false),
    semicolon_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false;false||true&&4<=3+2*1" => value(true),

    left_semicolon: ";1" => error(MissingOperand@0),
    both_semicolon: ";1;" => error(MissingOperand@0),

    semicolon_left_space: "1 ;2" => value(2),

    left_double_semicolon: ";;1" => error(MissingOperand@0),
    right_double_semicolon: "1;;" => error(MissingOperand@2),
    both_double_semicolon: ";;1;;" => error(MissingOperand@0),
    between_double_semicolon: "1;;2" => error(MissingOperand@2),

    paren_semicolon_all_over: ";(;(;););" => error(MissingOperand@0),
}
