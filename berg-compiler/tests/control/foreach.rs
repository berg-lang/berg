use crate::*;

#[test]
fn foreach_tuple() {
    expect("
        :sum = 0
        multiply: :x * :y
        foreach 1,2,3,4 {
            sum += multiply(:x, 2)
        }
        sum
    ").to_yield(20);
}

#[test]
fn foreach_single() {
    expect("
        :sum = 0
        foreach 1 { sum += :_ }
        sum
    ").to_yield(1);
}

#[test]
fn foreach_variable() {
    expect("
        :sum = 0
        :values = 1,2,3
        foreach values {
            sum += :_
        }
        sum
    ").to_yield(6);
}

#[test]
fn foreach_input_error() {
    expect("
        :sum = 0
        foreach 1/0 {
            sum += :_
        }
        sum
    ").to_error(DivideByZero, "0".line(3));
}

#[test]
fn foreach_second_input_error() {
    expect("
        :sum = 0
        foreach (1,{1/0}) {
            sum += :_
        }
        sum
    ").to_error(DivideByZero, "0".line(3));
}

#[test]
fn foreach_block_error() {
    expect("
        :sum = 0
        foreach (1,{1/0}) {
            sum += :_
            1/0
        }
        sum
    ").to_error(DivideByZero, "0".line(5));
}
