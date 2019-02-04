use crate::*;

#[test]
fn function() {
    expect("Square: :X*X; Square 10").to_yield(100)
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
