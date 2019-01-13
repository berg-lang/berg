pub mod compiler_test;
use compiler_test::*;

#[test] fn parens()                             { expect( "(1+2*3)*3"   ).to_yield(21) }
#[test] fn parens_neg()                         { expect( "-(1+2*3)*3"  ).to_yield(-21) }
#[test] fn parens_neg_neg()                     { expect( "-(-1+2*3)*3" ).to_yield(-15) }

#[test] fn outer_parens_number()                { expect( "(1)"         ).to_yield(1) }
#[test] fn outer_parens_add()                   { expect( "(1+2)"       ).to_yield(3) }
#[test] fn nested_parens()                      { expect( "((1))"       ).to_yield(1) }
#[test] fn nested_parens_add()                  { expect( "((1+2))"     ).to_yield(3) }
#[test] fn nested_parens_with_right()           { expect( "((1+2)*3)*4" ).to_yield(36) }
#[test] fn nested_parens_with_left()            { expect( "5*(6*(1+2))" ).to_yield(90) }
#[test] fn nested_parens_with_both()            { expect( "5*(6+(1+2)+3)+4" ).to_yield(64) }
#[test] fn nested_parens_with_neg()             { expect( "-(-(1+2))"   ).to_yield(3) }
#[test] fn nested_parens_with_neg_between()     { expect( "(-(1+2))"    ).to_yield(-3) }

#[test] fn empty_parens()                       { expect( "()"          ).to_yield(Nothing) }
#[test] fn nested_empty_parens()                { expect( "(())"        ).to_yield(Nothing) }
#[test] fn add_empty_parens_left()              { expect( "()+1"        ).to_error(UnsupportedOperator,2) }
#[test] fn add_empty_parens_right()             { expect( "1+()"        ).to_error(BadType,2..=3) }
#[test] fn add_empty_parens_both()              { expect( "()+()"       ).to_error(UnsupportedOperator,2) }
#[test] fn neg_empty_parens()                   { expect( "-()"         ).to_error(UnsupportedOperator,0) }

#[test] fn outer_parens_missing_operand_error() { expect( "(+)"         ).to_error(MissingOperand,1) }

#[test] fn missing_close_paren()                { expect( "("           ).to_error(OpenWithoutClose,0) }
#[test] fn missing_open_paren()                 { expect( ")"           ).to_error(CloseWithoutOpen,0) }
#[test] fn nested_empty_parens_missing_close()  { expect( "(()"         ).to_error(OpenWithoutClose,0) }
#[test] fn nested_empty_parens_missing_open()   { expect( "())"         ).to_error(CloseWithoutOpen,2) }
#[test] fn nested_empty_parens_missing_both_closes() { expect( "))"          ).to_error(CloseWithoutOpen,1) }
#[test] fn nested_empty_parens_missing_both_opens() { expect( "(("          ).to_error(OpenWithoutClose,0) }
