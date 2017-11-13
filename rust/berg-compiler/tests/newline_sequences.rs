#[macro_use]
pub mod compiler_test;

compiler_tests! {
    left_newline_1: "\n1" => type(1),
    right_newline_1: "1\n" => type(1),
    both_newline_1: "\n1\n" => type(1),

    left_double_newline_1: "\n\n1" => type(1),
    right_double_newline_1: "1\n\n" => type(1),
    both_double_newline_1: "\n\n1\n\n" => type(1),

    paren_newline_all_over: "\n(\n(\n)\n)\n" => type(nothing),

    newline_sequence: "1\n2" => type(2),
    newline_sequence_add: "1+1\n2+2" => type(4),
    newline_sequence_or_and_ge_plus_mul: "1*2+3>=4&&true||false\nfalse||true&&4>=3+2*1" => type(false),
    newline_sequence_or_and_le_plus_mul: "1*2+3<=4&&true||false\nfalse||true&&4<=3+2*1" => type(true),
}
