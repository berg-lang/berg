#[macro_use]
pub mod compiler_test;
use compiler_test::*;

compiler_tests! {
    field_access: "a = { :x = 10 }; a.x" => value(10),
    multiple_field_access: "a = { :x = 10; :y = 20 }; a.x + a.y" => value(30),
    nested_field_access: "a = { :x = { :y = 10 } }; a.x.y" => value(10),
    nested_field_access_from_nested_field: "a = { :x = { :y = 10 } }; b = { :c = { :d = a.x.y } }; b.c.d" => value(10),
    late_field_access: "a = { :x = 10 }; b = { a.x }; a = { :x = 20 }; b" => value(20),

    nested_parent_field_access_error: "a = { :x = { :y = 10 } }; a.x.x" => error(NoSuchPublicField@[26-30]),
    nested_child_field_access_error: "a = { :x = { :y = 10 } }; a.y" => error(NoSuchPublicField@[26-28]),
    self_field_access_error: "a = { :x = 10 }; a.a" => error(NoSuchPublicField@[17-19]),
    no_such_field_access: "a = { :x = 10 }; a.y" => error(NoSuchPublicField@[17-19]),
    private_field_access: "a = { x = 10 }; a.x" => error(PrivateField@[16-18]),
    primitive_field_access: "a = 10; a.x" => error(NoSuchPublicField@[8-10]),
    parent_field_access_error: ":x = 10; a = { y = 20 }; a.x" => error(NoSuchPublicField@[25-27]),

    // TODO make circular field access work right
    // circular_field_access_error: "x = { :y = { x.y } }" => error(CircularDependency@1),
    // roundabout_circular_field_access_error: "a = { :y = 10; }; x = { :y = { a.y } }; a = x; a" => error(CircularDependency@1),
}
