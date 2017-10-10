#[macro_use]
pub mod compiler_test;
use std::str::FromStr;

compiler_tests! {
    zero: "0" => result(0),
    one: "1" => result(1),
    huge: "999999999999999999999999999999999999999999999" => result(BigInt::from_str("999999999999999999999999999999999999999999999").unwrap()),
}
