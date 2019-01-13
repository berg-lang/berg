pub mod compiler_test;
use compiler_test::*;

#[test]
fn left_space_1()                      { expect( " 1"          ).to_yield(1) }
#[test]
fn right_space_1()                     { expect( "1 "          ).to_yield(1) }
#[test]
fn both_space_1()                      { expect( " 1 "         ).to_yield(1) }

#[test]
fn left_double_space_1()               { expect( "  1"         ).to_yield(1) }
#[test]
fn right_double_space_1()              { expect( "1  "         ).to_yield(1) }
#[test]
fn both_double_space_1()               { expect( "  1  "       ).to_yield(1) }

#[test]
fn paren_space_all_over()              { expect( " ( ( ) ) "   ).to_yield(Nothing) }

#[test]
fn addmul_paren_space_precedence()     { expect( "1+(2 * 3)"   ).to_yield(7) }
#[test]
fn addmul_paren_space_precedence_2()   { expect( "(1+2) * 3"   ).to_yield(9) }

#[test]
fn addmul_space_precedence()           { expect( "1+2 * 3"     ).to_yield(9) }
#[test]
fn addmuladd_space_precedence()        { expect( "1+2 * 3+4"   ).to_yield(21) }
#[test]
fn addmuladd_space_precedence_2()      { expect( "1 + 2*3 + 4" ).to_yield(11) }
#[test]
fn muladdmul_space_precedence()        { expect( "1 * 2+3 * 4" ).to_yield(20) }
#[test]
fn muladdmul_space_precedence_2()      { expect( "1*2 + 3*4"   ).to_yield(14) }

// Ensure precedence with newline works like with space
#[test]
fn addmul_paren_newline_precedence()   { expect( "1+(2\n*\n3)" ).to_yield(7) }
#[test]
fn addmul_paren_newline_precedence_2() { expect( "(1+2)\n*\n3" ).to_yield(9) }

#[test]
fn addmul_newline_precedence()         { expect( "1+2\n*\n3"   ).to_yield(9) }
#[test]
fn addmuladd_newline_precedence()      { expect( "1+2\n*\n3+4" ).to_yield(21) }
#[test]
fn addmuladd_newline_precedence_2()    { expect( "1\n+\n2*3\n+\n4" ).to_yield(11) }
#[test]
fn muladdmul_newline_precedence()      { expect( "1\n*\n2+3\n*\n4" ).to_yield(20) }
#[test]
fn muladdmul_newline_precedence_2()    { expect( "1*2\n+\n3*4" ).to_yield(14) }
