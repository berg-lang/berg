#[macro_use]
pub mod compiler_test;

compiler_tests! {
    true_literal: "true" => type(true),
    false_literal: "false" => type(false),
    uppercase_true: "TRUE" => error(NoSuchField@[0-3]) type(error),
    uppercase_false: "FALSE" => error(NoSuchField@[0-4]) type(error),
}
