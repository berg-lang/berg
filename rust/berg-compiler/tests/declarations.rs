#[macro_use]
pub mod compiler_test;

compiler_tests! {
    declaration_only: ":a" => type(nothing),
    reference_without_definition: ":a;a" => type(error) error(PropertyNotSet@3),
}
