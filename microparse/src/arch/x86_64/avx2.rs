#![allow(clippy::missing_safety_doc)]

use std::{arch::x86_64::*, simd::Simd};

pub const SIMD_BITS: usize = 256;
pub type SimdU8 = std::simd::Simd<u8, { 256 / 8 / size_of::<u8>() }>;

pub use super::sse4_2::prefix_xor;

#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    _mm256_shuffle_epi8(lookup_table.into(), keys.into()).into()
}

#[inline(always)]
pub const fn splat16(val: Simd<u8, 16>) -> SimdU8 {
    let val = val.to_array();
    SimdU8::from_array([
        val[0], val[1], val[2], val[3],
        val[4], val[5], val[6], val[7],
        val[8], val[9], val[10], val[11],
        val[12], val[13], val[14], val[15],

        val[0], val[1], val[2], val[3],
        val[4], val[5], val[6], val[7],
        val[8], val[9], val[10], val[11],
        val[12], val[13], val[14], val[15],
    ])
}