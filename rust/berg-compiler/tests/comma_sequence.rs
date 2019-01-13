pub mod compiler_test;
use compiler_test::*;

#[test]
fn comma_sequence()       { expect( "1,2"         ).to_yield(1) }//[1,2]) }
// #[test]
// fn comma_left_space()     { expect( "1 ,2"        ).to_yield([1,2]) }
// #[test]
// fn comma_right_space()    { expect( "1, 2"        ).to_yield([1,2]) }
// #[test]
// fn comma_both_space()     { expect( "1 , 2"       ).to_yield([1,2]) }

// #[test]
// fn comma_sequence_add()   { expect( "1+1+1,2+2+2" ).to_yield([3,6]) }
// #[test]
// fn comma_sequence_or_and_ge_plus_mul() { expect( "1*2+3>=4&&true||false,false||true&&4>=3+2*1" ).to_yield([true,false]) }
// #[test]
// fn comma_sequence_or_and_le_plus_mul() { expect( "1*2+3<=4&&true||false,false||true&&4<=3+2*1" ).to_yield([false,true]) }

#[test]
fn right_comma()          { expect( "1,"          ).to_error(MissingOperand,1) }
#[test]
fn left_comma()           { expect( ",1"          ).to_error(MissingOperand,0) }
#[test]
fn both_comma()           { expect( ",1,"         ).to_error(MissingOperand,2) }

#[test]
fn left_double_comma()    { expect( ",,1"         ).to_error(MissingOperand,0) }
#[test]
fn right_double_comma()   { expect( "1,,"         ).to_error(MissingOperand,2) }
#[test]
fn both_double_comma()    { expect( ",,1,,"       ).to_error(MissingOperand,4) }
#[test]
fn between_double_comma() { expect( "1,,2"        ).to_error(MissingOperand,1) }

#[test]
fn paren_comma_all_over() { expect( ",(,(,),),"   ).to_error(MissingOperand,8) }
