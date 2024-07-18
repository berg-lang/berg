pub use super::sse4_2::prefix_xor;

pub const SIMD_BITS: usize = 512;
pub type SimdU8 = std::simd::Simd<u8, { 512 / 8 / size_of::<u8>() }>;
