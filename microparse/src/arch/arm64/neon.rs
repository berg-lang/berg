#![allow(clippy::missing_safety_doc)]

use core::arch::aarch64::*;

pub const SIMD_BITS: usize = 128;
pub type SimdU8 = std::simd::Simd<u8, { 128 / 8 / size_of::<u8>() }>;

#[cfg(all(target_feature = "neon", target_feature = "aes"))]
#[inline]
pub unsafe fn prefix_xor(bitmask: u64) -> u64 {
    unsafe { vmull_p64(bitmask, ALL) as u64 }
}

#[target_feature(enable = "neon")]
pub unsafe fn lookup_lower16_ascii(lookup_table: SimdU8, keys: SimdU8) -> SimdU8 {
    // Mask out the high bits, except the utf8 one--we want to resolve high numbers to 0, just like
    // Intel
    let keys = keys & Simd::splat(0b1000_1111);
    vqtbl1q_u8(lookup_table.into(), keys.into()).into()
}

pub const fn splat16(val: Simd<u8, 16>) -> SimdU8 {
    val
}