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

macro_rules! assert_next_eq {
    ( $( $scanner:ident.next($input:expr) => $expected:expr );*) => ($({
        let input = backslashes($input);
        let actual = input.with_mask($scanner.next(input.into()));
        let expected = non_spaces($expected);
        assert_eq!(actual, expected, "Masks not equal!\n|     actual | {} |\n|   expected | {} |\n| difference | {} |", actual, expected, (*actual ^ *expected).fmt_ch('^', ' '));
    });*);
}

#[test]
fn zeroes() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_next_eq!(
        scanner.next(br"                                                                ")
                  => br"                                                                ";
        scanner.next(br"                                                                ")
                  => br"                                                                ";
        scanner.next(br"                                                                ")
                  => br"                                                                "
    );
}

#[test]
fn all() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_next_eq!(
        scanner.next(br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\")
                  => br" \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\";
        scanner.next(br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\")
                  => br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\";
        scanner.next(br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\")
                  => br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\";
        scanner.next(br"                                                                ")
                  => br"\                                                               "
    );
}

#[test]
fn overflow_only() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_next_eq!(
        scanner.next(br"                                                               \")
                  => br"                                                                ";
        scanner.next(br"                                                               \")
                  => br"\                                                               ";
        scanner.next(br"                                                                ")
                  => br"\                                                               "
    );
}

#[test]
fn all_except_overflow() {
    let mut scanner: PrecededByScanner = Default::default();
    assert_next_eq!(
        scanner.next(br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\ ")
                  => br" \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\";
        scanner.next(br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\ ")
                  => br" \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\";
        scanner.next(br"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\ ")
                  => br" \\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\";
        scanner.next(br"                                                                ")
                  => br"                                                                "
    );
}

}