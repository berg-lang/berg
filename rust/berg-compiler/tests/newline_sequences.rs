#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    left_newline: "\n1" => value(1),
    right_newline: "1\n" => value(1),
    both_newline: "\n1\n" => value(1),

    left_double_newline: "\n\n1" => value(1),
    right_double_newline: "1\n\n" => value(1),
    both_double_newline: "\n\n1\n\n" => value(1),
    double_newline_between: "1\n\n2" => value(2),

    paren_newline_all_over: "\n(\n(\n)\n)\n" => value(Nothing),

    newline_sequence: "1\n2" => value(2),
    newline_sequence_add: "1+1\n2+2" => value(4),
    newline_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false\nfalse||true&&4>=3+2*1" => value(false),
    newline_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false\nfalse||true&&4<=3+2*1" => value(true),
}
