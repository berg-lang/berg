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
    lazy_reference_to_lazy_scope: "a = 1; b = { a + 2 }; c = { b + 4 }; c" => value(7),
    nested_lazy_block: "{ { 1 } }" => value(1),
    nested_field_access: "a = 1; { b = 2; { c = 3; a } }" => value(1),
    lazy_field_never_executed: "a = 1; b = { a += 2 }; a" => value(1),
    lazy_block_closes_over_scope: "a = 1; { b = 2; a = { b + 4 } }; a" => value(6),
    lazy_block_closes_over_scope_writeable: "a = 1; x = { b = 2; c = { b += a; b }; c }; x" => value(3),
    lazy_block_assigned_and_used_out_of_scope: "a = 1; x = { b = 2; c = { b + a }; c }; y = { a + x }; y" => value(4),
}
