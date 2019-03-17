use crate::*;

#[test]
fn function() {
    expect("Square: :x*x; Square 10").to_yield(100)
}
#[test]
fn function_multiple_args() {
    expect("Multiply: :a*:b; Multiply(2,3)").to_yield(6)
}
#[test]
fn function_multiple_args_space() {
    expect("Multiply: :a*:b; Multiply 2,3").to_yield(6)
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
