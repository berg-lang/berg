pub mod compiler_test;
use crate::compiler_test::*;

#[test]
fn comma_sequence() {
    expect("1,2").to_yield(tuple!(1, 2))
}
#[test]
fn comma_sequence_three() {
    expect("1,2,3").to_yield(tuple!(1, 2, 3))
}
#[test]
fn comma_left_space() {
    expect("1 ,2").to_yield(tuple!(1, 2))
}
#[test]
fn comma_right_space() {
    expect("1, 2").to_yield(tuple!(1, 2))
}
#[test]
fn comma_both_space() {
    expect("1 , 2").to_yield(tuple!(1, 2))
}

#[test]
fn comma_sequence_add() {
    expect("1+1+1,2+2+2").to_yield(tuple!(3, 6))
}
#[test]
fn comma_sequence_or_and_ge_plus_mul() {
    expect("1*2+3>=4&&true||false,false||true&&4>=3+2*1").to_yield(tuple!(true, false))
}
#[test]
fn comma_sequence_or_and_le_plus_mul() {
    expect("1*2+3<=4&&true||false,false||true&&4<=3+2*1").to_yield(tuple!(false, true))
}

#[test]
fn comma_sequence_bare_parentheses() {
    expect("(1,2)").to_yield(tuple!(1,2));
}

#[test]
fn only_comma() {
    expect(",").to_error(MissingOperand, 0)
}
#[test]
fn right_comma() {
    expect("1,").to_yield(tuple!(1))
}
#[test]
fn right_comma_nested() {
    expect("(1),").to_yield(tuple!(1))
}
#[test]
fn right_comma_inside_parentheses() {
    expect("(1,)").to_yield(tuple!(1))
}
#[test]
fn right_comma_inside_and_outside_parentheses() {
    expect("(1,),").to_yield(tuple!([1]))
}
#[test]
fn right_comma_nested_multiple() {
    expect("(1,2),").to_yield(tuple!([1,2]))
}
#[test]
fn left_comma() {
    expect(",1").to_error(MissingOperand, 0)
}
#[test]
fn both_comma() {
    expect(",1,").to_error(MissingOperand, 0)
}

#[test]
fn left_double_comma() {
    expect(",,1").to_error(MissingOperand, 0)
}
#[test]
fn right_double_comma() {
    expect("1,,").to_error(MissingOperand, 1)
}
#[test]
fn both_double_comma() {
    expect(",,1,,").to_error(MissingOperand, 0)
}
#[test]
fn between_double_comma() {
    expect("1,,2").to_error(MissingOperand, 1)
}

#[test]
fn paren_comma_all_over() {
    expect(",(,(,),),").to_error(MissingOperand, 0)
}

#[test]
fn nested_comma() {
    expect("1,(2,3)").to_yield(tuple!(1, [2,3]));
}

#[test]
fn nested_comma_first() {
    expect("(1,2),3").to_yield(tuple!([1,2], 3));
}

#[test]
fn nested_comma_single() {
    expect("(1,2),").to_yield(tuple!([1,2]));
}

#[test]
#[should_panic]
fn comma_trailing_comma_is_not_same_as_nested_tuple() {
    expect("(1,2),").to_yield(tuple!(1,2));
}

#[test]
fn comma_trailing_comma_is_same_as_single_value() {
    expect("1,").to_yield(1);
}

#[test]
#[should_panic]
fn comma_multiple_values_is_not_same_as_nested_tuple() {
    expect("1,2").to_yield(tuple!([1,2]));
}

#[test]
#[should_panic]
fn comma_bare_parentheses_is_not_same_as_nested_tuple() {
    expect("(1,2)").to_yield(tuple!([1,2]));
}
