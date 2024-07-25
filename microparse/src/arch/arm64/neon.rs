#![allow(clippy::missing_safety_doc)]

use core::arch::aarch64::*;

pub use crate::arch::define_simd::simd128::*;

#[target_feature(enable = "neon,aes")]
#[inline]
pub const unsafe fn prefix_xor(bitmask: u64) -> u64 {
    unsafe { vmull_p64(bitmask, ALL) as u64 }
}

#[target_feature(enable = "neon")]
#[inline(always)]
pub const unsafe fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    // Mask out the high bits, except the utf8 one--we want to resolve high numbers to 0, just like
    // Intel
    let keys = keys & Simd::splat(0b1000_1111);
    vqtbl1q_u8(lookup_table.into(), keys.into()).into()
}

const MERGED_MASK_BITMASK: SimdU8 = Simd::from_array([
    0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80,
    0x01, 0x02, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80
]);

#[inline(always)]
pub fn into_merged_mask(masks: [SimdMask8; SIMD_CHUNKS64_COUNT]) -> crate::primitive_mask::Mask64 {
      // Add each of the elements next to each other, successively, to stuff each 8 byte mask into one.
      let mut sum0 = (masks[0] & MERGED_MASK_BITMASK) + (masks[1] & MERGED_MASK_BITMASK);
      let sum1 = (masks[2] & MERGED_MASK_BITMASK) + (masks[3] & MERGED_MASK_BITMASK);
      sum0 += sum1;
      sum0 += sum0;
      sum0[0]
}

const NTH_BIT: SimdU8 = std::simd::Simd::from_array([
    0x01u8, 0x02u8, 0x4u8, 0x8u8, 0x10u8, 0x20u8, 0x40u8, 0x80u8,
    0x01u8, 0x02u8, 0x4u8, 0x8u8, 0x10u8, 0x20u8, 0x40u8, 0x80u8
]);

#[inline(always)]
pub fn merge_to_bitmask(masks: impl IntoUnwrappingIterator<SIMD_PER_64_BYTES, UnwrappingItem=SimdMask8>) -> crate::primitive_mask::Mask64 {
    let mut masks = masks.into_unwrapping_iter();
    // Add each of the elements next to each other, successively, to stuff each 8 byte mask into one.
    let mask0 = masks.next().to_bitmask_vector() & NTH_BIT;
    let mask1 = masks.next().to_bitmask_vector() & NTH_BIT;
    let mut sum0 = mask0 + mask1;
    let mask2 = masks.next().to_bitmask_vector() & NTH_BIT;
    let mask3 = masks.next().to_bitmask_vector() & NTH_BIT;
    let sum1 = mask2 + mask3;
    sum0 += sum1;
    sum0 += sum0;
    let sum0: SimdU64 = unsafe { std::mem::transmute(sum0) };
    sum0[0]
}

/// ARM accumulates the mask in a SIMD register, because it has no direct movemask instruction.
pub struct Mask64Builder(pub SimdU8, pub SimdU8);
pub struct SimdMaskBuilder(SimdU8, SimdU8, SimdU8);

impl Mask64Builder {
    #[inline(always)]
    pub fn push<T: MaskElement, const N: usize>(&mut self, mask: Mask<T, N>, u64_lane: usize) where LaneCount<N>: SupportedLaneCount {
        // From simdjson:
        //   // Add each of the elements next to each other, successively, to stuff each 8 byte mask into one.
        //   uint8x16_t sum0 = vpaddq_u8(this->chunks[0] & bit_mask, this->chunks[1] & bit_mask);
        //   uint8x16_t sum1 = vpaddq_u8(this->chunks[2] & bit_mask, this->chunks[3] & bit_mask);
        //   sum0 = vpaddq_u8(sum0, sum1);
        //   sum0 = vpaddq_u8(sum0, sum0);
        //   return vgetq_lane_u64(vreinterpretq_u64_u8(sum0), 0);        
        let mask = mask.to_bitmask_vector() & NTH_BIT;
        match u64_lane {
            // self.0 = 0
            0 => self.0 = mask,

            // self.0 = 0+1
            1 => self.0 = vpaddq_u8(self.0.into(), mask.into()),

            // self.0 = 0+1
            // self.1 = 2
            2 => self.1 = mask,

            // self.0 = 0+1+2+3
            3 => {
                self.1 = vpaddq_u8(self.1.into(), mask.into());
                self.0 = vpaddq_u8(self.0.into(), second);
                // This is a sort of redundant operation that produces masks in both the lower and
                // upper half of self.0. This happens because we don't have another level of masks
                // to merge in!
                self.0 = vpaddq_u8(self.0.into(), self.0.into());
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn finish() -> Mask64 {
        self.0
    }
}

impl SimdMaskBuilder {
    #[inline(always)]
    pub fn push<T: MaskElement, const N: usize>(&mut self, mask: Mask<T, N>, u64_lane: usize) where LaneCount<N>: SupportedLaneCount {
        // From simdjson:
        //   // Add each of the elements next to each other, successively, to stuff each 8 byte mask into one.
        //   uint8x16_t sum0 = vpaddq_u8(this->chunks[0] & bit_mask, this->chunks[1] & bit_mask);
        //   uint8x16_t sum1 = vpaddq_u8(this->chunks[2] & bit_mask, this->chunks[3] & bit_mask);
        //   sum0 = vpaddq_u8(sum0, sum1);
        //   sum0 = vpaddq_u8(sum0, sum0);
        //   return vgetq_lane_u64(vreinterpretq_u64_u8(sum0), 0);        
        let mask = mask.to_bitmask_vector() & NTH_BIT;
        match u64_lane {
            // self.0 = 0
            0 => self.0 = mask,

            // self.0 = 0+1
            1 => self.0 = vpaddq_u8(self.0.into(), mask.into()),

            // self.0 = 0+1
            // self.1 = 2
            2 => self.1 = mask,

            // self.0 = 0+1+2+3
            3 => {
                self.1 = vpaddq_u8(self.1.into(), mask.into());
                self.0 = vpaddq_u8(self.0.into(), second);
            }

            // self.0 = 0+1+2+3
            // self.1 = 4
            4 => self.1 = mask,

            // self.0 = 0+1+2+3
            // self.1 = 4+5
            5 => self.1 = vpaddq_u8(self.1.into(), mask.into()),

            // self.0 = 0+1+2+3
            // self.1 = 4+5
            // self.2 = 6
            6 => self.2 = mask,

            // self.0 = 0+1+2+3+4+5+6+7
            7 => {
                self.2 = vpaddq_u8(self.2.into(), mask.into());
                self.1 = vpaddq_u8(self.1.into(), self.2.into());
                self.0 = vpaddq_u8(self.0.into(), self.1.into());
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn finish() -> SimdU8 {
        self.0
    }
}
