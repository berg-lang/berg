#[macro_use]
pub mod compiler_test;
use std::str::FromStr;

compiler_tests! {
    zero: "0" => type(0),
    one: "1" => type(1),
    huge: "999999999999999999999999999999999999999999999" => type(BigRational::from_str("999999999999999999999999999999999999999999999").unwrap()),
}
