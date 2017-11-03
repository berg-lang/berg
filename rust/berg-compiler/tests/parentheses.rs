#[macro_use]
pub mod compiler_test;

compiler_tests! {
    parens: "(1+2*3)*3" => type(21),
    parens_neg: "-(1+2*3)*3" => type(-21),
    parens_neg_neg: "-(-1+2*3)*3" => type(-15),

    outer_parens_number: "(1)" => type(1),
    outer_parens_add: "(1+2)" => type(3),
    nested_parens: "((1))" => type(1),
    nested_parens_add: "((1+2))" => type(3),
    nested_parens_with_right: "((1+2)*3)*4" => type(36),
    nested_parens_with_left: "5*(6*(1+2))" => type(90),
    nested_parens_with_both: "5*(6+(1+2)+3)+4" => type(64),
    nested_parens_with_neg: "-(-(1+2))" => type(3),
    nested_parens_with_neg_between: "(-(1+2))" => type(-3),

    empty_parens: "()" => type(nothing),
    nested_empty_parens: "(())" => type(nothing),
    add_empty_parens_left: "()+1" => type(error),
    add_empty_parens_right: "1+()" => type(error),
    add_empty_parens_both: "()+()" => type(error),
    neg_empty_parens: "-()" => type(error),

    outer_parens_missing_operand_error: "(+)" => error(MissingRightOperand@1) type(error),

    missing_close_paren: "(" => error(OpenWithoutClose@0) type(nothing),
    missing_open_paren: ")" => error(CloseWithoutOpen@0) type(nothing),
    nested_empty_parens_missing_close: "(()" => error(OpenWithoutClose@0) type(nothing),
    nested_empty_parens_missing_open: "())" => error(CloseWithoutOpen@2) type(nothing),
    nested_empty_parens_missing_both_closes: "))" => errors(CloseWithoutOpen@0,CloseWithoutOpen@1) type(nothing),
    nested_empty_parens_missing_both_opens: "((" => errors(OpenWithoutClose@0,OpenWithoutClose@1) type(nothing),
}
