use super::{Mask64, BLOCK_SIZE};

#[repr(transparent)]
pub struct PrecededByScanner<const N: usize = 1> {
    pub matches: Mask64,
}

impl<const N: usize> Default for PrecededByScanner<N> {
    #[inline(always)]
    fn default() -> Self {
        Self { matches: 0 }
    }
}

impl<const N: usize> PrecededByScanner<N> {
    #[inline(always)]
    pub fn next(&mut self, matches: Mask64) -> Mask64 {
        const { assert!(N < BLOCK_SIZE); };
        let preceded_by = (matches << N) | (self.matches >> (BLOCK_SIZE-N));
        self.matches = matches; // Save it for next time around
        preceded_by
    }
}

#[cfg(test)]
mod tests {
use super::*;
use crate::primitive_mask::test::*;

macro_rules! assert_mask_eq {
    ($actual:expr, $expected:expr) => ({
        let actual = $actual;
        let expected = $expected;
        assert_eq!(actual, expected, "Masks not equal!\nactual    : {}\nexpected  : {}\ndifference: {}", actual.fmt_x(), expected.fmt_x(), (actual ^ expected).fmt_ch('^', ' '));
    })
}

#[test]
fn zeroes() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_mask_eq!(scanner.next(NONE), NONE);
    assert_mask_eq!(scanner.next(NONE), NONE);
    assert_mask_eq!(scanner.next(NONE), NONE);
}

#[test]
fn all() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_mask_eq!(scanner.next(ALL), !FIRST);
    assert_mask_eq!(scanner.next(ALL), ALL);
    assert_mask_eq!(scanner.next(ALL), ALL);
    assert_mask_eq!(scanner.next(NONE), FIRST);
}

#[test]
fn overflow_only() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_mask_eq!(scanner.next(LAST), NONE);
    assert_mask_eq!(scanner.next(LAST), FIRST);
    assert_mask_eq!(scanner.next(NONE), FIRST);
    assert_mask_eq!(scanner.next(NONE), NONE);
}

#[test]
fn all_except_overflow() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_mask_eq!(scanner.next(!LAST), !FIRST);
    assert_mask_eq!(scanner.next(!LAST), !FIRST);
    assert_mask_eq!(scanner.next(!LAST), !FIRST);
    assert_mask_eq!(scanner.next(NONE), NONE);
}

}