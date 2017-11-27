#[macro_use]
pub mod compiler_test;

compiler_tests! {
    right_semicolon: "1;" => type(nothing),
    semicolon_right_space: "1; 2" => type(2),
    semicolon_both_space: "1 ; 2" => type(2),
    semicolon_sequence: "1;2" => type(2),
    semicolon_sequence_add: "1+1;2+2" => type(4),
    semicolon_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false;false||true&&4>=3+2*1" => type(false),
    semicolon_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false;false||true&&4<=3+2*1" => type(true),

    left_semicolon: ";1" => errors(UnrecognizedOperator@0) type(error),
    both_semicolon: ";1;" => errors(UnrecognizedOperator@0) type(nothing),

    semicolon_left_space: "1 ;2" => errors(UnrecognizedOperator@2) type(error),

    left_double_semicolon: ";;1" => errors(MissingLeftOperand@0,UnrecognizedOperator@1) type(error),
    right_double_semicolon: "1;;" => errors(MissingLeftOperand@2) type(nothing),
    both_double_semicolon: ";;1;;" => errors(MissingLeftOperand@0,UnrecognizedOperator@1,MissingLeftOperand@4) type(nothing),
    between_double_semicolon: "1;;2" => errors(UnrecognizedOperator@2) type(error),

    paren_semicolon_all_over: ";(;(;););" => errors(UnrecognizedOperator@0,UnrecognizedOperator@2,MissingLeftOperand@4) type(nothing),
}
