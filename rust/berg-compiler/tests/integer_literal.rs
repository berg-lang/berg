#[macro_use]
pub mod compiler_test;
use compiler_test::berg_compiler::BigRational;
use std::str::FromStr;

compiler_tests! {
    zero: "0" => value(0),
    one: "1" => value(1),
    huge: "999999999999999999999999999999999999999999999" => value(BigRational::from_str("999999999999999999999999999999999999999999999").unwrap()),
}
