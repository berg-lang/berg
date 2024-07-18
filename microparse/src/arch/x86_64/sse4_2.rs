#![allow(clippy::missing_safety_doc)]

use core::arch::x86_64::*;
use std::simd::Simd;

pub const SIMD_BITS: usize = 128;
pub type SimdU8 = Simd<u8, { 128 / 8 / size_of::<u8>() }>;

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
pub unsafe fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    _mm_shuffle_epi8(lookup_table.into(), keys.into()).into()
}

pub const fn splat16(val: Simd<u8, 16>) -> SimdU8 {
    val
}