mod escape_scanner;
mod string_scanner;

use std::simd::Simd;

pub use escape_scanner::*;

pub type Mask64 = u64;
pub type Block64 = Simd<u8, 64>;
pub const CHUNK_LEN: usize = size_of::<Mask64>();
pub const ALL: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const NONE: u64 = 0x0000_0000_0000_0000;
pub const ODD_BITS: u64 = 0xAAAA_AAAA_AAAA_AAAA;

#[derive(Default, Debug, Clone)]
pub struct PrecededBy<const N: usize = 1> {
    pub prev_matches: Mask64,
}

impl<const N: usize> PrecededBy<N> {
    #[inline]
    pub fn next(&mut self, matches: Mask64) -> Mask64 {
        let result = matches >> N | self.prev_matches << (CHUNK_LEN-N);
        self.prev_matches = matches;
        result
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.prev_matches != 0
    }
}
