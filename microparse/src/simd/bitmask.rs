use std::simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount};

use generic::SupportedSimdWidth;

use crate::primitive_mask::Mask64;

use super::*;

#[repr(transparent)]
struct SimdBitmaskBuilder<W: SupportedSimdWidth>(pub W::Lanes64::Simd<u64>);

impl SimdBitmaskBuilder<2> {
    #[inline(always)]
    pub fn push(&mut self, mask: Mask<i8, 16>, u16_lane: usize) {
        // TODO don't 
        // TODO there might be a way to do this without pulling the bitmask 

        self.0 = match u16_lane {
            0 => mask,
            1 => self.0 + self.0 + mask + mask,
            2 => self.0 + mask,
            3 => self.0 + mask,
            _ => unreachable!(),
        };
    }

    #[inline(always)]
    pub fn push_mask(&mut self, mask: Mask64, u64_lane: usize) {
        self.0[u64_lane] = mask;
    }

    #[inline(always)]
    pub fn finish(self) -> Simd<u64, U64_LANES> {
        self.0
    }
}