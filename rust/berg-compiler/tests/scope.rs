pub mod compiler_test;
use compiler_test::*;

#[test] fn child_references_parent()                     { expect( "a = 1; { a + 2 }"                       ).to_yield(3) }
#[test] fn child_references_parent_too_early_error()     { expect( "{ a + 2 }; a = 1"                       ).to_error(NoSuchField,2) }
    // TODO make circular reference detection work again
// #[test] fn child_references_parent_circular_error()      { expect( "a = { a + 1 }; a"                       ).to_error(NoSuchField,6) }
// #[test] fn child_references_parent_lazy_circular_error() { expect( "a = 1; a = { a + 1 }; a"                ).to_error(CircularDependency,13..=13) }
#[test] fn child_defines_value_early()                   { expect( "x = { a = 1; a+2 }; a = x + 3; a"       ).to_yield(6) }
#[test] fn child_updates_parent()                        { expect( "a = 1; { a += 2 }; a + 3"               ).to_yield(6) }
#[test] fn child_overwrites_parent()                     { expect( "a = 1; { a = 2 }; a + 3"                ).to_yield(5) }
#[test] fn child_overwrites_and_references_parent()      { expect( "a = 1; { a = 2; a + 3 } + a"            ).to_yield(7) }
#[test] fn child_redefines_parent()                      { expect( "a = 1; { :a = 2 }; a"                   ).to_yield(1) }
#[test] fn parent_references_child_error()               { expect( "{ a = 0; a }; a"                        ).to_error(NoSuchField,14) }
#[test] fn parent_defines_value_late()                   { expect( "x = { a+2 }; a = 3; x"                  ).to_error(NoSuchField,6) }
#[test] fn child_evaluated_lazily()                      { expect( "a = 1; x = { a }; a = 2; x"             ).to_yield(2) }
#[test] fn lazy_reference_to_lazy_scope()                { expect( "a = 1; b = { a + 2 }; c = { b + 4 }; c" ).to_yield(7) }
#[test] fn nested_lazy_block()                           { expect( "{ { 1 } }"                              ).to_yield(1) }
#[test] fn nested_field_access()                         { expect( "a = 1; { b = 2; { c = 3; a } }"         ).to_yield(1) }
#[test] fn lazy_field_never_executed()                   { expect( "a = 1; b = { a += 2 }; a"               ).to_yield(1) }
#[test] fn lazy_block_closes_over_scope()                { expect( "a = 1; { b = 2; a = { b + 4 } }; a"     ).to_yield(6) }
#[test] fn lazy_block_closes_over_scope_writeable()      { expect( "a = 1; x = { b = 2; c = { b += a; b }; c }; x" ).to_yield(3) }
#[test] fn lazy_block_assigned_and_used_out_of_scope()   { expect( "a = 1; x = { b = 2; c = { b + a }; c }; y = { a + x }; y" ).to_yield(4) }
