#[macro_use]
pub mod compiler_test;

compiler_tests! {
    left_space_1: " 1" => type(1),
    right_space_1: "1 " => type(1),
    both_space_1: " 1 " => type(1),

    left_double_space_1: "  1" => type(1),
    right_double_space_1: "1  " => type(1),
    both_double_space_1: "  1  " => type(1),

    paren_space_all_over: " ( ( ) ) " => type(nothing),

    addmul_space_precedence: "1+2 * 3" => type(9),
    addmuladd_space_precedence: "1+2 * 3+4" => type(21),
    addmuladd_space_precedence_2: "1 + 2*3 + 4" => type(11),
    muladdmul_space_precedence: "1 * 2+3 * 4" => type(20),
    muladdmul_space_precedence_2: "1*2 + 3*4" => type(14),
}