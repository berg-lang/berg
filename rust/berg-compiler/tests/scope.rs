#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    child_references_parent: "a = 1; { a + 2 }" => value(3),
    child_references_parent_too_early_error: "{ a + 2 }; a = 1" => error(NoSuchField@2),
    // TODO make circular reference detection work again
    // child_references_parent_circular_error: "a = { a + 1 }; a" => error(NoSuchField@6),
    // child_references_parent_lazy_circular_error: "a = 1; a = { a + 1 }; a" => error(CircularDependency@13),
    child_defines_value_early: "x = { a = 1; a+2 }; a = x + 3; a" => value(6),
    child_updates_parent: "a = 1; { a += 2 }; a + 3" => value(6),
    child_overwrites_parent: "a = 1; { a = 2 }; a + 3" => value(5),
    child_overwrites_and_references_parent: "a = 1; { a = 2; a + 3 } + a" => value(7),
    child_redefines_parent: "a = 1; { :a = 2 }; a" => value(1),
    parent_references_child_error: "{ a = 0; a }; a" => error(NoSuchField@14),
    parent_defines_value_late: "x = { a+2 }; a = 3; x" => error(NoSuchField@6),
    child_evaluated_lazily: "a = 1; x = { a }; a = 2; x" => value(2),
}
