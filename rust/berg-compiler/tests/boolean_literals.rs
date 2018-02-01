#[macro_use]
pub mod compiler_test;
use compiler_test::*;

// #[test]
// fn true_literal() {
//     expect("true").to_yield(true)
// }
compiler_tests! {
    true_literal: "true" => value(true),
    false_literal: "false" => value(false),
    uppercase_true: "TRUE" => error(NoSuchField@[0-3]),
    uppercase_false: "FALSE" => error(NoSuchField@[0-4]),
}
