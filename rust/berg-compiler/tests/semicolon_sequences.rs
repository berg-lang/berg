pub mod compiler_test;
use compiler_test::*;

#[test] fn right_semicolon()          { expect( "1;"          ).to_yield(Nothing) }
#[test] fn semicolon_right_space()    { expect( "1; 2"        ).to_yield(2) }
#[test] fn semicolon_both_space()     { expect( "1 ; 2"       ).to_yield(2) }
#[test] fn semicolon_sequence()       { expect( "1;2"         ).to_yield(2) }
#[test] fn semicolon_sequence_add()   { expect( "1+1+1;2+2+2" ).to_yield(6) }
#[test] fn semicolon_sequence_or_and_ge_plus_mul() { expect( "1*2+3>=4&&true||false;false||true&&4>=3+2*1" ).to_yield(false) }
#[test] fn semicolon_sequence_or_and_le_plus_mul() { expect( "1*2+3<=4&&true||false;false||true&&4<=3+2*1" ).to_yield(true) }

#[test] fn left_semicolon()           { expect( ";1"          ).to_error(MissingOperand,0) }
#[test] fn both_semicolon()           { expect( ";1;"         ).to_error(MissingOperand,0) }

#[test] fn semicolon_left_space()     { expect( "1 ;2"        ).to_yield(2) }

#[test] fn left_double_semicolon()    { expect( ";;1"         ).to_error(MissingOperand,0) }
#[test] fn right_double_semicolon()   { expect( "1;;"         ).to_error(MissingOperand,2) }
#[test] fn both_double_semicolon()    { expect( ";;1;;"       ).to_error(MissingOperand,0) }
#[test] fn between_double_semicolon() { expect( "1;;2"        ).to_error(MissingOperand,2) }

#[test] fn paren_semicolon_all_over() { expect( ";(;(;););"   ).to_error(MissingOperand,0) }
