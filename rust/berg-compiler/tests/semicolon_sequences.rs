#[macro_use]
pub mod compiler_test;

compiler_tests! {
    left_semicolon: ";1" => errors(MissingLeftOperand@0) type(1),
    right_semicolon: "1;" => type(nothing),
    both_semicolon: ";1;" => errors(MissingLeftOperand@0) type(nothing),

    semicolon_right_space: "1; 2" => type(2),
    semicolon_left_space: "1 ;2" => type(2),
    semicolon_both_space: "1 ;2" => type(2),

    left_double_semicolon: ";;1" => errors(MissingLeftOperand@0,MissingLeftOperand@1) type(1),
    right_double_semicolon: "1;;" => errors(MissingLeftOperand@2) type(nothing),
    both_double_semicolon: ";;1;;" => errors(MissingLeftOperand@0,MissingLeftOperand@1,MissingLeftOperand@4) type(nothing),
    between_double_semicolon: "1;;2" => errors(MissingLeftOperand@2) type(2),

    paren_semicolon_all_over: ";(;(;););" => errors(MissingLeftOperand@0,MissingLeftOperand@2,MissingLeftOperand@4) type(nothing),

    semicolon_sequence: "1;2" => type(2),
    semicolon_sequence_add: "1+1;2+2" => type(4),
    semicolon_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false;false||true&&4>=3+2*1" => type(false),
    semicolon_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false;false||true&&4<=3+2*1" => type(true),
}
