#![allow(clippy::missing_safety_doc)]

use core::arch::x86_64::*;
use crate::unwrapping_iterator::IntoUnwrappingIterator;

pub use crate::arch::define_simd::simd128::*;
pub use crate::primitive_mask::mask64::Mask64Builder;

///
/// Compute a prefix xor of the bitmask: turn a bit on if it's preceded by an even number of 1's,
/// and off if it's preceded by an odd number of 1's.
/// 
#[target_feature(enable="sse4.2,pclmulqdq")]
#[inline]
pub unsafe fn prefix_xor(bitmask: u64) -> u64 {
    unsafe {
        let all_ones = _mm_set1_epi64x(u64::MAX as i64);
        let result = _mm_clmulepi64_si128(_mm_set_epi64x(0i64, bitmask as i64), all_ones, 0);
        _mm_cvtsi128_si64(result) as u64
    }
}

#[target_feature(enable="sse4.2")]
#[inline]
pub unsafe fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    _mm_shuffle_epi8(lookup_table.into(), keys.into()).into()
}

#[inline(always)]
pub fn merge_to_bitmask(masks: impl IntoUnwrappingIterator<SIMD_PER_64_BYTES, UnwrappingItem=SimdMask8>) -> crate::primitive_mask::Mask64 {
    let mut masks = masks.into_unwrapping_iter();
    masks.next().to_bitmask()
    | (masks.next().to_bitmask() << SIMD_BYTES)
    | (masks.next().to_bitmask() << (SIMD_BYTES*2))
    | (masks.next().to_bitmask() << (SIMD_BYTES*3))
}
