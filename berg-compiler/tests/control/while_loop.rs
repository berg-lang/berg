use crate::*;

#[test]
fn while_1_through_5() {
    expect(":x = 1; while { x <= 5 } { x++ }; x").to_yield(6);
}

#[test]
fn while_condition_error_exits_early() {
    expect(":x = 1; :y = 1; while { x/y <= 5 } { x++; y = 0 }; x").to_error(DivideByZero, 26);
}

#[test]
fn while_block_error_exits_early() {
    expect(":x = 1; :y = 1; while { x <= 5 } { x/y; x++; y = 0 }; x").to_error(DivideByZero, 37);
}

#[test]
fn while_int_condition_bad_type() {
    expect(":x = 1; while { 1 } { x++ }; x").to_error(BadOperandType, 14..=18);
}

#[test]
fn while_empty_condition_bad_type() {
    expect(":x = 1; while {} { x++ }; x").to_error(BadOperandType, 14..=15);
}

#[test]
fn while_non_block_condition() {
    expect(":x = 1; while ( x <= 5 ) { x++ }; x").to_error(WhileConditionMustBeBlock, 14..=23);
}

#[test]
fn while_non_block_block() {
    expect(":x = 1; while { x <= 5 } ( x++ ); x").to_error(WhileBlockMustBeBlock, 25..=31);
}

#[test]
fn while_missing_condition() {
    expect(":x = 1; while; x").to_error(WhileWithoutCondition, 8..=12);
}

#[test]
fn while_missing_block() {
    expect(":x = 1; while { x <= 5 }; x").to_error(WhileWithoutBlock, 8..=12);
}

#[test]
fn add_while_1() {
    expect("while + 1").to_error(WhileWithoutCondition, 0..=4)
}

#[test]
fn add_1_while() {
    expect("1 + while").to_error(WhileWithoutCondition, 4..=8)
}

#[test]
fn add_1_while_block() {
    expect(":x = 1; 1 + while { x <= 5 } { x++ }").to_error(WhileWithoutCondition, 12..=16)
}
