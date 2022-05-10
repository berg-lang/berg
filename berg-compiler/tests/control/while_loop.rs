use crate::*;

#[test]
fn while_1_through_5() {
    expect(
        "
        :x = 1
        while { x <= 5 } {
            x++
        }
        x
    ",
    )
    .to_yield(6);
}

#[test]
fn while_condition_error_exits_early() {
    expect(
        "
        :x = 1
        :y = 1
        while { x/y <= 5 } {
            x++
            y = 0
        }
        x
    ",
    )
    .to_error(DivideByZero, "y".within("x/y"));
}

#[test]
fn while_block_error_exits_early() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x/y
            x++
            y = 0
        }
        x
    ",
    )
    .to_error(DivideByZero, "y".within("x/y"));
}

#[test]
fn while_int_condition_bad_type() {
    expect(
        "
        :x = 1
        while { 1 } {
            x++
        }
        x
    ",
    )
    .to_error(BadOperandType, "{ 1 }");
}

#[test]
fn while_empty_condition_bad_type() {
    expect(
        "
        :x = 1
        while {} {
            x++
        }
        x
    ",
    )
    .to_error(BadOperandType, "{}");
}

#[test]
fn while_non_block_condition() {
    expect(
        "
        :x = 1
        while ( x <= 5 ) {
            x++
        }
        x
    ",
    )
    .to_error(WhileConditionMustBeBlock, "( x <= 5 )");
}

#[test]
fn while_non_block_block() {
    expect(
        "
        :x = 1
        while { x <= 5 } ( x++ )
        x
    ",
    )
    .to_error(WhileBlockMustBeBlock, "( x++ )");
}

#[test]
fn while_missing_condition() {
    expect(":x = 1; while; x").to_error(WhileWithoutCondition, "while");
}

#[test]
fn while_missing_block() {
    expect(
        "
        :x = 1
        while { x <= 5 }
        x
    ",
    )
    .to_error(WhileWithoutBlock, "while");
}

#[test]
fn add_while_1() {
    expect("while + 1").to_error(WhileWithoutCondition, "while")
}

#[test]
fn add_1_while() {
    expect("1 + while").to_error(WhileWithoutCondition, "while")
}

#[test]
fn add_1_while_block() {
    expect(
        "
        :x = 1
        1 + while { x <= 5 } { x++ }
    ",
    )
    .to_error(WhileWithoutCondition, "while")
}

#[test]
fn break_exits_while() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            break
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(2, 1))
}

#[test]
fn break_exits_while_from_nested_block() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            { break }
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(2, 1))
}
#[test]
fn break_exits_while_from_if() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            if x >= 3 { break }
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(3, 2))
}
#[test]
fn break_exits_while_from_callback() {
    expect(
        "
        :DoThisIf = { (:cond,:arg); if cond { arg } else { } }
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            DoThisIf { x >= 3 }, { break }
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(3, 2))
}

#[test]
fn continue_skips_remaining_block() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            continue
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(6, 1))
}

#[test]
fn continue_skips_remaining_block_from_nested_block() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            { continue }
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(6, 1))
}
#[test]
fn continue_skips_remaining_block_from_if() {
    expect(
        "
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            if x >= 3 { continue }
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(6, 2))
}
#[test]
fn continue_skips_remaining_block_from_callback() {
    expect(
        "
        :DoThisIf = { (:cond,:arg); if cond { arg } else { } }
        :x = 1
        :y = 1
        while { x <= 5 } {
            x++
            DoThisIf { x >= 3 }, { continue }
            y++
        }
        x,y
    ",
    )
    .to_yield(tuple!(6, 2))
}

#[test]
fn dangling_break() {
    expect("break").to_error(BreakOutsideLoop, "break")
}

#[test]
fn dangling_continue() {
    expect("continue").to_error(ContinueOutsideLoop, "continue")
}
