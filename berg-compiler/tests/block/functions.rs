use crate::*;

#[test]
fn function_no_args() {
    expect("Ten: 10; Ten()").to_yield(10)
}
#[test]
fn function_single_arg_space() {
    expect("Square: :x*x; Square 10").to_yield(100)
}
#[test]
fn function_single_arg_trailing_comma() {
    expect("Square: :x*x; Square 10,").to_yield(100)
}
#[test]
fn function_single_arg_paren() {
    expect("Square: :x*x; Square(10)").to_yield(100)
}
#[test]
fn function_single_arg_space_paren() {
    expect("Square: :x*x; Square (10)").to_yield(100)
}
#[test]
fn function_single_arg_block() {
    expect("Square: :x*x; Square { 10 }").to_yield(100)
}
#[test]
fn function_single_tuple_arg() {
    expect("Duplicate: (:a,a); Duplicate (1,2)").to_yield(tuple!([1,2],[1,2]))
}
#[test]
fn function_single_empty_tuple_arg() {
    expect("Duplicate: (:a,a); Duplicate ()").to_yield(tuple!([],[]))
}
#[test]
fn function_single_block_tuple_arg() {
    expect("Duplicate: (:a,a); Duplicate { 1,2 }").to_yield(tuple!((1,2),(1,2)))
}
#[test]
fn function_multiple_args_space() {
    expect("Multiply: :a*:b; Multiply 2,3").to_yield(6)
}
#[test]
fn function_multiple_args_trailing_comma() {
    expect("Multiply: :a*:b; Multiply 2,3,").to_yield(6)
}
#[test]
fn function_multiple_args_paren() {
    expect("Multiply: :a*:b; Multiply(2,3)").to_yield(6)
}
#[test]
fn function_result() {
    expect("F: { x: 1; y: 2; x+y }; F").to_yield(3)
}
#[test]
fn function_result_add() {
    expect("F: { x: 1; y: 2; x+y }; F + 2").to_yield(5)
}
#[test]
fn function_field() {
    expect("F: { x: 1; y: 2; x+y }; F.x").to_yield(1)
}
#[test]
fn function_result_and_field() {
    expect("F: { x: 1; y: 2; x+y }; F + F.x").to_yield(4)
}
#[test]
fn functions_evaluate_immediately() {
    expect(":x = 1; F = { x = :y }; :oldX = x; F(2); oldX,x").to_yield(tuple!(1, 2))
}
#[test]
fn functions_evaluate_all_levels_immediately() {
    expect(":x = 1; F = { x = :y; { x = y + 1; } }; :oldX = x; F(2); oldX,x").to_yield(tuple!(1, 3))
}
