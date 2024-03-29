use crate::*;

#[test]
fn if_true_no_else_runs_correct_blocks() {
    expect("if true { 1 }").to_yield(1);
    expect(
        "
        :a = 0
        if (a += 1; true) {
            a += 2
        }
        a
    ",
    )
    .to_yield(1 + 2);
}
#[test]
fn if_false_no_else_runs_correct_blocks() {
    expect("if false { 1 }").to_yield(tuple!());
    expect(
        "
        :a = 0
        if (a += 1; false) {
            a += 2
        }
        a
    ",
    )
    .to_yield(1);
}
#[test]
fn if_true_else_runs_correct_blocks() {
    expect(
        "
        if true {
            1
        } else {
            2
        }
    ",
    )
    .to_yield(1);
    // expect("
    //     :a = 0
    //     if (a += 1; true) {
    //         a += 2
    //     } else {
    //         a += 4
    //     }
    //     a
    // ").to_yield(1+2);
}
#[test]
fn if_false_else_runs_correct_block() {
    expect("if false { 1 } else { 2 }").to_yield(2);
    expect(
        "
        :a = 0
        if (a += 1; false) {
            a += 2
        } else {
            a += 4
        }
        a
    ",
    )
    .to_yield(1 + 4);
}
#[test]
fn if_true_if_true_else_runs_correct_blocks_and_berg_does_it_wrong() {
    expect("if true { 1 } else if true { 2 } else { 3 }").to_yield(1);
    expect(
        "
        :a = 0
        if (a += 1; true) {
            a += 2
        } else if (a += 4; true) {
            a += 8
        } else {
            a += 16
        }
        a
    ",
    )
    .to_yield(1 + 2 + 4);
}
#[test]
fn if_true_if_false_else_runs_correct_blocks_and_berg_does_it_wrong() {
    expect("if true { 1 } else if false { 2 } else { 3 }").to_yield(1);
    expect(
        "
        :a = 0
        if (a += 1; true) {
            a += 2
        } else if (a += 4; false) {
            a += 8
        } else {
            a += 16
        }
        a
    ",
    )
    .to_yield(1 + 2 + 4);
}
#[test]
fn if_false_if_false_else_runs_correct_blocks() {
    expect("if false { 1 } else if false { 2 } else { 3 }").to_yield(3);
    expect(
        "
        :a = 0
        if (a += 1; false) {
            a += 2
        } else if (a += 4; false) {
            a += 8
        } else {
            a += 16
        }
        a
    ",
    )
    .to_yield(1 + 4 + 16);
}
#[test]
fn if_false_if_true_else_runs_correct_blocks() {
    expect("if false { 1 } else if true { 2 } else { 3 }").to_yield(2);
    expect(
        "
        :a = 0
        if (a += 1; false) {
            a += 2
        } else if (a += 4; true) {
            a += 8
        } else {
            a += 16
        }
        a
    ",
    )
    .to_yield(1 + 4 + 8);
}

#[test]
fn if_int_bad_type_error() {
    expect("if 1 { 10 }").to_error(BadOperandType, "1")
}
#[test]
fn if_empty_bad_type_error() {
    expect("if () { 10 }").to_error(BadOperandType, "()")
}
#[test]
fn else_if_empty_bad_type_error() {
    expect("if false {} else if () { 10 }").to_error(BadOperandType, "()")
}

#[test]
fn dangling_if_error() {
    expect("if").to_error(IfWithoutCondition, "if");
}
#[test]
fn dangling_else_error() {
    expect("else").to_error(ElseWithoutIf, "else");
}
#[test]
fn dangling_if_if_error() {
    expect("if if").to_error(IfWithoutCondition, "if");
}
#[test]
fn dangling_if_else_error() {
    expect("if else").to_error(IfWithoutCondition, "if");
}
#[test]
fn dangling_if_true_if_error() {
    expect("if true if").to_error(IfBlockMustBeBlock, "if".after("true"));
}
#[test]
fn dangling_if_true_else_error() {
    expect("if true else").to_error(IfBlockMustBeBlock, "else");
}
#[test]
fn dangling_if_false_if_error() {
    expect("if false if").to_error(IfBlockMustBeBlock, "if".after("false"));
}
#[test]
fn dangling_if_false_else_error() {
    expect("if false else").to_error(IfBlockMustBeBlock, "else");
}
#[test]
fn dangling_if_true_block_if_error() {
    expect("if true {} if").to_error(IfFollowedByNonElse, "if".after("true"));
}
#[test]
fn dangling_if_true_block_else_error() {
    expect("if true {} else").to_error(ElseWithoutBlock, "else");
}
#[test]
fn dangling_if_false_block_if_error() {
    expect("if false {} if").to_error(IfFollowedByNonElse, "if".after("false"));
}
#[test]
fn dangling_if_false_block_else_error() {
    expect("if false {} else").to_error(ElseWithoutBlock, "else");
}
#[test]
fn dangling_if_true_block_else_if_error() {
    expect("if true {} else if").to_error(IfWithoutCondition, "if".after("else"));
}
#[test]
fn dangling_if_true_block_else_else_error() {
    expect("if true {} else else").to_error(ElseBlockMustBeBlock, "else".after("else"));
}
#[test]
fn dangling_if_false_block_else_if_error() {
    expect("if false {} else if").to_error(IfWithoutCondition, "if".after("else"));
}
#[test]
fn dangling_if_false_block_else_else_error() {
    expect("if false {} else else").to_error(ElseBlockMustBeBlock, "else".after("else"));
}
#[test]
fn dangling_if_true_block_else_if_if_error() {
    expect("if true {} else if if").to_error(IfWithoutCondition, "if".after("else"));
}
#[test]
fn dangling_if_true_block_else_if_else_error() {
    expect("if true {} else if else").to_error(IfWithoutCondition, "if".after("else"));
}
#[test]
fn dangling_if_false_block_else_if_if_error() {
    expect("if false {} else if if").to_error(IfWithoutCondition, "if".after("else"));
}
#[test]
fn dangling_if_false_block_else_if_else_error() {
    expect("if false {} else if else").to_error(IfWithoutCondition, "if".after("else"));
}
#[test]
fn if_true_without_block_error() {
    expect("if true").to_error(IfWithoutBlock, "if")
}
#[test]
fn if_false_without_block_error() {
    expect("if false").to_error(IfWithoutBlock, "if")
}
#[test]
fn if_true_block_else_if_true_without_block_error() {
    expect("if true {} else if true").to_error(IfWithoutBlock, "if true {} else if")
}
#[test]
fn if_true_block_else_if_false_without_block_error() {
    expect("if true {} else if false").to_error(IfWithoutBlock, "if true {} else if")
}
#[test]
fn if_false_block_else_if_true_without_block_error() {
    expect("if false {} else if true").to_error(IfWithoutBlock, "if false {} else if")
}
#[test]
fn if_false_block_else_if_false_without_block_error() {
    expect("if false {} else if false").to_error(IfWithoutBlock, "if false {} else if")
}

#[test]
fn add_if_1() {
    expect("if + 1").to_error(IfWithoutCondition, "if")
}
#[test]
fn add_else_1() {
    expect("else + 1").to_error(ElseWithoutIf, "else")
}
#[test]
fn add_1_if() {
    expect("1 + if").to_error(IfWithoutCondition, "if")
}
#[test]
fn add_1_if_block() {
    expect("1 + if true { 2 }").to_error(IfWithoutCondition, "if")
}
#[test]
fn add_1_else() {
    expect("1 + else").to_error(ElseWithoutIf, "else")
}

#[test]
fn if_runs_block_lazily() {
    expect(
        "
        :a = 0
        :b = (
            if (a += 1; true) {
                a += 2
                a
            }
        )
        a,b,{ a }
    ",
    )
    .to_yield(tuple!(1, 3, 3));
}
#[test]
fn else_runs_block_lazily() {
    expect(
        "
        :a = 0
        :b = (
            if (a += 1; false) {
                a += 2
                a
            } else {
                a += 4
                a
            }
        )
        a,b,{ a }
    ",
    )
    .to_yield(tuple!(1, 5, 5));
}
#[test]
fn else_if_runs_block_lazily() {
    expect(
        "
        :a = 0
        :b = (
            if (a += 1; false) {
                a += 2
                a
            } else if (a += 4; true) {
                a += 8
                a
            }
        )
        a,b,{ a }
    ",
    )
    .to_yield(tuple!(5, 13, 13));
}

#[test]
fn if_scope_impacts_parent_but_this_test_demonstrates_berg_is_not_doing_it_right() {
    expect("if true { :a = 10 } else { :b = 10 }; a").to_error(NoSuchField, 38); //.to_yield(10);
    expect("if true { :a = 10 } else { :b = 10 }; b").to_error(NoSuchField, 38); //.to_error(FieldNotSet, 39);
    expect("if false { :a = 10 } else { :b = 10 }; a").to_error(NoSuchField, 39); //.to_error(FieldNotSet, 39);
    expect("if false { :a = 10 } else { :b = 10 }; b").to_error(NoSuchField, 39);
    //.to_yield(10);
}
