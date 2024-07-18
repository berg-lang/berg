#![allow(clippy::missing_safety_doc)]

use core::arch::aarch64::*;

pub const SIMD_BITS: usize = 128;
pub type SimdU8 = std::simd::Simd<u8, { 128 / 8 / size_of::<u8>() }>;

#[cfg(all(target_feature = "neon", target_feature = "aes"))]
#[inline]
pub unsafe fn prefix_xor(bitmask: u64) -> u64 {
    unsafe { vmull_p64(bitmask, ALL) as u64 }
}

