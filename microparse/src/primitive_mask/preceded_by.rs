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
        let preceded_by = (matches >> N) | (self.matches << (BLOCK_SIZE-N));
        self.matches = matches; // Save it for next time around
        preceded_by
    }
}
