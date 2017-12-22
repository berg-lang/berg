#[macro_use]
pub mod compiler_test;

compiler_tests! {
    left_space_1: " 1" => value(1),
    right_space_1: "1 " => value(1),
    both_space_1: " 1 " => value(1),

    left_double_space_1: "  1" => value(1),
    right_double_space_1: "1  " => value(1),
    both_double_space_1: "  1  " => value(1),

    paren_space_all_over: " ( ( ) ) " => value(nothing),

    addmul_paren_space_precedence: "1+(2 * 3)" => value(7),
    addmul_paren_space_precedence_2: "(1+2) * 3" => value(9),

    addmul_space_precedence: "1+2 * 3" => value(9),
    addmuladd_space_precedence: "1+2 * 3+4" => value(21),
    addmuladd_space_precedence_2: "1 + 2*3 + 4" => value(11),
    muladdmul_space_precedence: "1 * 2+3 * 4" => value(20),
    muladdmul_space_precedence_2: "1*2 + 3*4" => value(14),

    // Ensure precedence with newline works like with space
    addmul_paren_newline_precedence: "1+(2\n*\n3)" => value(7),
    addmul_paren_newline_precedence_2: "(1+2)\n*\n3" => value(9),

    addmul_newline_precedence: "1+2\n*\n3" => value(9),
    addmuladd_newline_precedence: "1+2\n*\n3+4" => value(21),
    addmuladd_newline_precedence_2: "1\n+\n2*3\n+\n4" => value(11),
    muladdmul_newline_precedence: "1\n*\n2+3\n*\n4" => value(20),
    muladdmul_newline_precedence_2: "1*2\n+\n3*4" => value(14),
}