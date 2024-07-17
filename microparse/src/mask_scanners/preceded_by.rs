use super::{Mask64, BLOCK_SIZE};

#[repr(transparent)]
pub struct PrecededByScanner<const N: usize = 1> {
    pub prev_matches: Mask64,
}

impl<const N: usize> PrecededByScanner<N> {
    #[inline]
    pub fn next(&mut self, matches: Mask64) -> Mask64 {
        let result = matches >> N | self.prev_matches << (BLOCK_SIZE-N);
        self.prev_matches = matches;
        result
    }
}
