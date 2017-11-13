#[macro_use]
pub mod compiler_test;

compiler_tests! {
    left_newline: "\n1" => type(1),
    right_newline: "1\n" => type(1),
    both_newline: "\n1\n" => type(1),

    left_double_newline: "\n\n1" => type(1),
    right_double_newline: "1\n\n" => type(1),
    both_double_newline: "\n\n1\n\n" => type(1),
    double_newline_between: "1\n\n2" => type(2),

    paren_newline_all_over: "\n(\n(\n)\n)\n" => type(nothing),

    newline_sequence: "1\n2" => type(2),
    newline_sequence_add: "1+1\n2+2" => type(4),
    newline_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false\nfalse||true&&4>=3+2*1" => type(false),
    newline_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false\nfalse||true&&4<=3+2*1" => type(true),
}
