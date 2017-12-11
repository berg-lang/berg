#[macro_use]
pub mod compiler_test;

compiler_tests! {
    child_references_parent: "a = 1; { a + 2 }" => type(3),
    child_references_parent_too_early_error: "{ a + 2 }; a = 1" => error(NoSuchField@2) type(nothing),
    child_references_parent_container_error: "a = { a + 1 }" => error(NoSuchField@6) type(nothing),
    child_defines_value_early: "x = { a = 1; a+2 }; a = x + 3; a" => type(6),
    child_updates_parent: "a = 1; { a += 2 }; a + 3" => type(6),
    child_overwrites_parent: "a = 1; { a = 2 }; a + 3" => type(5),
    child_overwrites_and_references_parent: "a = 1; { a = 2; a + 3 } + a" => type(7),
    parent_references_child_error: "{ a = 0; a }; a" => error(NoSuchField@14) type(error),
}
