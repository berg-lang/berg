use crate::*;

#[test]
fn field_access() {
    expect("a = { :x = 10 }; a.x").to_yield(10)
}
#[test]
fn multiple_field_access() {
    expect("a = { :x = 10; :y = 20 }; a.x + a.y").to_yield(30)
}
#[test]
fn nested_field_access() {
    expect("a = { :x = { :y = 10 } }; a.x.y").to_yield(10)
}
#[test]
fn nested_field_access_from_nested_field() {
    expect("a = { :x = { :y = 10 } }; b = { :c = { :d = a.x.y } }; b.c.d").to_yield(10)
}
#[test]
fn late_field_access() {
    expect("a = { :x = 10 }; b = { a.x }; a = { :x = 20 }; b").to_yield(20)
}

#[test]
fn nested_parent_field_access_error() {
    expect("a = { :x = { :y = 10 } }; a.x.x").to_error(NoSuchPublicField, 30)
}
#[test]
fn nested_child_field_access_error() {
    expect("a = { :x = { :y = 10 } }; a.y").to_error(NoSuchPublicField, 28)
}
#[test]
fn self_field_access_error() {
    expect("a = { :x = 10 }; a.a").to_error(NoSuchPublicField, 19)
}
#[test]
fn no_such_field_access() {
    expect("a = { :x = 10 }; a.y").to_error(NoSuchPublicField, 19)
}
#[test]
fn private_field_access() {
    expect("a = { x = 10 }; a.x").to_error(PrivateField, 18)
}
#[test]
fn primitive_field_access() {
    expect("a = 10; a.x").to_error(NoSuchPublicField, 10)
}
#[test]
fn parent_field_access_error() {
    expect(":x = 10; a = { y = 20 }; a.x").to_error(NoSuchPublicField, 27)
}

#[test]
fn set_field() {
    expect("a = { :x = 1 }; a.x = 10; a.x").to_yield(10)
}
#[test]
fn multiple_set_field() {
    expect("a = { :x = 1; :y = 2 }; a.x = 10; a.y = 20; a.x + a.y").to_yield(30)
}
#[test]
fn nested_set_field() {
    expect("a = { :b = { :x = 1 } }; a.b.x = 10; a.b.x").to_yield(10)
}
#[test]
fn set_no_such_field() {
    expect("a = { :x }; a.y = 20; a.y").to_error(NoSuchPublicField, 14)
}
#[test]
fn private_set_field() {
    expect("a = { x = 10 }; a.x = 20; a.x").to_error(PrivateField, 18)
}
#[test]
fn primitive_set_field() {
    expect("a = 10; a.x = 10; a.x").to_error(NoSuchPublicField, 10)
}
#[test]
fn parent_field_set_error() {
    expect(":x = 10; a = { :y = 20 }; a.x = 30; a.x").to_error(NoSuchPublicField, 28)
}
// TODO make circular field access work right
// #[test]
// fn circular_field_access_error()      { expect( "x = { :y = { x.y } }; x.y"                     ).to_error(CircularDependency,1) }
// #[test]
// fn roundabout_circular_field_access_error() { expect( "a = { :y = 10; }; x = { :y = { a.y } }; a = x; a" ).to_error(CircularDependency,1) }
