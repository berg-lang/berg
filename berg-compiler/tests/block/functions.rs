use crate::*;

#[test]
fn function() {
    expect("Square: :X*X; Square 10").to_yield(100)
}
#[test]
fn function_multiple_args() {
    expect("Multiply: :A*:B; Multiply(2,3)").to_yield(6)
}
#[test]
fn function_multiple_args_space() {
    expect("Multiply: :A*:B; Multiply 2,3").to_yield(6)
}
#[test]
fn function_result() {
    expect("F: { X: 1; Y: 2; X+Y }; F").to_yield(3)
}
#[test]
fn function_result_add() {
    expect("F: { X: 1; Y: 2; X+Y }; F + 2").to_yield(5)
}
#[test]
fn function_field() {
    expect("F: { X: 1; Y: 2; X+Y }; F.X").to_yield(1)
}
#[test]
fn function_result_and_field() {
    expect("F: { X: 1; Y: 2; X+Y }; F + F.X").to_yield(4)
}
#[test]
fn functions_evaluate_immediately() {
    expect(":X = 1; F = { X = :Y }; :OldX = X; F(2); OldX,X").to_yield(tuple!(1,2))
}
#[test]
fn functions_evaluate_only_one_level_immediately() {
    expect(":X = 1; F = { X = :Y; { X = Y + 10 } }; :OldX = X; F(2); OldX,X").to_yield(tuple!(1,2))
}
