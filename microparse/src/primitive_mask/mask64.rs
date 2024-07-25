use std::simd::{LaneCount, Mask, MaskElement, SupportedLaneCount};

pub type Mask64 = u64;
pub const ALL: u64 = 0xFFFF_FFFF_FFFF_FFFF;
pub const NONE: u64 = 0x0000_0000_0000_0000;
pub const ODD: u64 = 0xAAAA_AAAA_AAAA_AAAA;
pub const EVEN: u64 = 0x5555_5555_5555_5555;
pub const LAST: u64 = 0x8000_0000_0000_0000;
pub const FIRST: u64 = 0x0000_0000_0000_0001;

#[repr(transparent)]
pub struct Mask64Builder(pub Mask64);

impl Mask64Builder {
    #[inline(always)]
    pub fn push<T: MaskElement, const N: usize>(&mut self, mask: Mask<T, N>, u64_lane: usize) where LaneCount<N>: SupportedLaneCount {
        self.0 |= mask.to_bitmask() << (u64_lane*N);
    }

    #[inline(always)]
    pub fn push_primitive<T: Into<Mask64>>(&mut self, mask: T, u64_lane: usize) {
        let lane_width = size_of::<T>()*8;
        let bit_index = u64_lane*lane_width;
        assert!(bit_index+lane_width <= 64);
        self.0 |= mask.into() << bit_index;
    }

    // Push part of a full u64 primitive mask
    #[inline(always)]
    pub fn push_partial_primitive(&mut self, mask: Mask64, u64_lane: usize, lane_width: usize) {
        let bit_index = u64_lane*lane_width;
        assert!(bit_index+lane_width <= 64);
        // Verify that the mask only contains the bits we asked for
        assert!(0 != mask & (((1 << lane_width) - 1) << bit_index));
        self.0 |= mask << (u64_lane*lane_width);
    }

    pub fn finish(self) -> Mask64 {
        self.0
    }
}

impl From<Mask64Builder> for Mask64 {
    #[inline(always)]
    fn from(builder: Mask64Builder) -> Self {
        builder.0
    }
}
