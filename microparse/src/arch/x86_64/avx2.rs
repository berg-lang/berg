pub use super::sse4_2::prefix_xor;

pub const SIMD_BITS: usize = 256;
pub type SimdU8 = std::simd::Simd<u8, { 256 / 8 / size_of::<u8>() }>;
