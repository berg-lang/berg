#![allow(clippy::missing_safety_doc)]

use std::arch::x86_64::*;
use crate::unwrapping_iterator::IntoUnwrappingIterator;

pub use crate::arch::define_simd::simd512::*;
pub use crate::primitive_mask::mask64::Mask64Builder;
pub use super::sse4_2::prefix_xor;


#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    _mm512_shuffle_epi8(lookup_table.into(), keys.into()).into()
}

#[inline(always)]
pub fn merge_to_bitmask(masks: impl IntoUnwrappingIterator<SIMD_PER_64_BYTES, UnwrappingItem=SimdMask8>) -> crate::primitive_mask::Mask64 {
    let mut masks = masks.into_unwrapping_iter();
    masks.next().to_bitmask()
}
