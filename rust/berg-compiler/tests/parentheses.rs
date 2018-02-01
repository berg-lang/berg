#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    parens: "(1+2*3)*3" => value(21),
    parens_neg: "-(1+2*3)*3" => value(-21),
    parens_neg_neg: "-(-1+2*3)*3" => value(-15),

    outer_parens_number: "(1)" => value(1),
    outer_parens_add: "(1+2)" => value(3),
    nested_parens: "((1))" => value(1),
    nested_parens_add: "((1+2))" => value(3),
    nested_parens_with_right: "((1+2)*3)*4" => value(36),
    nested_parens_with_left: "5*(6*(1+2))" => value(90),
    nested_parens_with_both: "5*(6+(1+2)+3)+4" => value(64),
    nested_parens_with_neg: "-(-(1+2))" => value(3),
    nested_parens_with_neg_between: "(-(1+2))" => value(-3),

    empty_parens: "()" => value(Nothing),
    nested_empty_parens: "(())" => value(Nothing),
    add_empty_parens_left: "()+1" => error(UnsupportedOperator@2),
    add_empty_parens_right: "1+()" => error(BadType@[2-3]),
    add_empty_parens_both: "()+()" => error(UnsupportedOperator@2),
    neg_empty_parens: "-()" => error(UnsupportedOperator@0),

    outer_parens_missing_operand_error: "(+)" => error(MissingOperand@1),

    missing_close_paren: "(" => error(OpenWithoutClose@0),
    missing_open_paren: ")" => error(CloseWithoutOpen@0),
    nested_empty_parens_missing_close: "(()" => error(OpenWithoutClose@0),
    nested_empty_parens_missing_open: "())" => error(CloseWithoutOpen@2),
    nested_empty_parens_missing_both_closes: "))" => error(CloseWithoutOpen@1),
    nested_empty_parens_missing_both_opens: "((" => error(OpenWithoutClose@0),
}
