use crate::*;

#[test]
fn try_catch() {
    expect("
        try { 1/0 } catch { :error.ErrorCode }
    ").to_yield(DivideByZero as usize);
}
