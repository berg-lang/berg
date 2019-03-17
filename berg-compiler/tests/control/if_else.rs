use crate::*;

#[test]
fn if_true_no_else_runs_correct_blocks() {
    expect("if true { 1 }").to_yield(1);
    expect(":a = 0; if (a += 1; true) { a += 2 }; a").to_yield(1+2);
}
#[test]
fn if_false_no_else_runs_correct_blocks() {
    expect("if false { 1 }").to_yield(tuple!());
    expect(":a = 0; if (a += 1; false) { a += 2 }; a").to_yield(1);
}
#[test]
fn if_true_else_runs_correct_blocks() {
    expect("if true { 1 } else { 2 }").to_yield(1);
    expect(":a = 0; if (a += 1; true) { a += 2 } else { a += 4 }; a").to_yield(1+2);
}
#[test]
fn if_false_else_runs_correct_block() {
    expect("if false { 1 } else { 2 }").to_yield(2);
    expect(":a = 0; if (a += 1; false) { a += 2 } else { a += 4 }; a").to_yield(1+4);
}
#[test]
fn if_true_if_true_else_runs_correct_blocks_and_berg_does_it_wrong() {
    expect("if true { 1 } else if true { 2 } else { 3 }").to_yield(1);
    expect(":a = 0; if (a += 1; true) { a += 2 } else if (a += 4; true) { a += 8 } else { a += 16 }; a").to_yield(1+2+4);
}
#[test]
fn if_true_if_false_else_runs_correct_blocks_and_berg_does_it_wrong() {
    expect("if true { 1 } else if false { 2 } else { 3 }").to_yield(1);
    expect(":a = 0; if (a += 1; true) { a += 2 } else if (a += 4; false) { a += 8 } else { a += 16 }; a").to_yield(1+2+4);
}
#[test]
fn if_false_if_false_else_runs_correct_blocks() {
    expect("if false { 1 } else if false { 2 } else { 3 }").to_yield(3);
    expect(":a = 0; if (a += 1; false) { a += 2 } else if (a += 4; false) { a += 8 } else { a += 16 }; a").to_yield(1+4+16);
}
#[test]
fn if_false_if_true_else_runs_correct_blocks() {
    expect("if false { 1 } else if true { 2 } else { 3 }").to_yield(2);
    expect(":a = 0; if (a += 1; false) { a += 2 } else if (a += 4; true) { a += 8 } else { a += 16 }; a").to_yield(1+4+8);
}

#[test]
fn dangling_if_error() {
    expect("if").to_error(IfWithoutCondition, 0..=1);
}
#[test]
fn dangling_if_if_error() {
    expect("if if").to_error(IfWithoutCondition, 3..=4);
}
#[test]
fn dangling_if_true_if_error() {
    expect("if true if").to_error(IfWithoutCondition, 8..=9);
}
#[test]
fn dangling_if_false_if_error() {
    expect("if false if").to_error(IfWithoutCode, 0..=1);
}
#[test]
fn dangling_if_true_block_if_error() {
    expect("if true {} if").to_error(IfWithoutElse, 11..=12);
}
#[test]
fn dangling_if_false_block_if_error() {
    expect("if false {} if").to_error(IfWithoutElse, 12..=13);
}
#[test]
fn dangling_if_true_block_else_if_error() {
    expect("if true {} else if").to_error(IfWithoutCondition, 16..=17);
}
#[test]
fn dangling_if_false_block_else_if_error() {
    expect("if false {} else if").to_error(IfWithoutCondition, 17..=18);
}
#[test]
fn dangling_if_true_block_else_if_if_error_yields_wrong_error_range_because_of_current_limitations_in_berg() {
    expect("if true {} else if if").to_error(IfWithoutCondition, 0..=17);
}
#[test]
fn dangling_if_false_block_else_if_if_error() {
    expect("if false {} else if if").to_error(IfWithoutCondition, 20..=21);
}
// #[test]
// fn if_true_without_block_error() {
//     expect("if true").to_error(IfWithoutCode, 0..=1)
// }
// #[test]
// fn if_false_without_block_error() {
//     expect("if true").to_error(IfWithoutCode, 0..=1)
// }
// #[test]
// fn else_only_error() {
//     expect("else").to_error(ElseWithoutIf, 0..=3)
// }
// #[test]
// fn else_after_else_error() {
//     expect("if false {} else else").to_error(ElseWithoutIf, 17..=20)
// }
// #[test]
// fn else_after_condition_error() {
//     expect("if false else").to_error(ElseWithoutIf, 9..=12)
// }

#[test]
fn if_runs_block_lazily() {
    expect(":a = 0; :b = (if (a += 1; true) { a += 2; a }); a,b,{ a }").to_yield(tuple!(1,3,3));
}
#[test]
fn else_runs_block_lazily() {
    expect(":a = 0; :b = (if (a += 1; false) { a += 2; a } else { a += 4; a }); a,b,{ a }").to_yield(tuple!(1,5,5));
}
#[test]
fn else_if_runs_block_lazily() {
    expect(":a = 0; :b = (if (a += 1; false) { a += 2; a } else if (a += 4; true) { a += 8; a }); a,b,{ a }").to_yield(tuple!(5,13,13));
}

#[test]
fn if_scope_impacts_parent_but_this_test_demonstrates_berg_is_not_doing_it_right()
{
    expect("if true { :a = 10 } else { :b = 10 }; a").to_error(NoSuchField, 38);//.to_yield(10);
    expect("if true { :a = 10 } else { :b = 10 }; b").to_error(NoSuchField, 38);//.to_error(FieldNotSet, 39);
    expect("if false { :a = 10 } else { :b = 10 }; a").to_error(NoSuchField, 39);//.to_error(FieldNotSet, 39);
    expect("if false { :a = 10 } else { :b = 10 }; b").to_error(NoSuchField, 39);//.to_yield(10);
}
