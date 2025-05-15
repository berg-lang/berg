use std::simd::{LaneCount, Mask, Simd, SupportedLaneCount};
use super::constants::SimdConstants;

// A B C D E F G H
// AB  CD  EF  GH
// ABCD    EFGH
// ABCDEFGH

// 88888888 4444 22 1
//                  A
//               AB
//               AB C
//          ABCD
//          ABCD    E
//          ABCD EF
//          ABCD EF G
// ABCDEFGH


trait HorizontalAdd {
    fn horizontal_add(a: Self, b: Self) -> Self;
}
cfg_if::cfg_if! {
    if #[cfg(target_feature = "neon")] {
        impl HorizontalAdd for Simd<u8, 128> {
            fn horizontal_add(a: Self, b: Self) -> Self {
                core::arch::aarch64::vpaddq_u8(a.into(), b.into())
            }
         }
    } else if #[cfg(target_feature = "sse3")] {
        impl HorizontalAdd for Simd<u8, 128> {
            fn horizontal_add(a: Self, b: Self) -> Self {
                core::arch::aarch64::vpaddq_u8(a.into(), b.into())
            }
        }
    }
}

//      A[N/2] B[N/2]
// Lets you add 8 SIMD-sized bytemasks together to produce 1 SIMD-sized bitmask
struct SimdMaskBuilder<const N: usize> where LaneCount<N>: SupportedLaneCount {
    step: usize,
    cache_abcd: Simd<u8, N>,
    cache_cef: Simd<u8, N>,
    cache_g: Simd<u8, N>,
}

impl<const N: usize> Default for SimdMaskBuilder<N> where LaneCount<N>: SupportedLaneCount, [u8; N]: SimdConstants {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> From<SimdMaskBuilder<N>> for Simd<u8, N> where LaneCount<N>: SupportedLaneCount, [u8; N]: SimdConstants {
    fn from(builder: SimdMaskBuilder<N>) -> Self {
        builder.finish()
    }
}

impl<const N: usize> SimdMaskBuilder<N> where LaneCount<N>: SupportedLaneCount, [u8; N]: SimdConstants {
    const NTH_BIT: Simd<u8, N> = Simd::from_array(<[u8; N] as SimdConstants>::NTH_BIT);

    pub fn new() -> Self {
        Self {
            step: 0,
            cache_abcd: Simd::default(),
            cache_cef: Simd::default(),
            cache_g: Simd::default(),
        }
    }

    pub fn append(&mut self, mask: Mask<i8, N>) {
        let simd = mask.to_bitmask_vector() & Self::NTH_BIT;
        match self.step {
            0 => self.cache_abcd = simd,
            1 => self.cache_abcd = horizontal_add(self.cache_abcd, simd),
            2 => self.cache_cef  = simd,
            3 => self.cache_abcd = horizontal_add(self.cache_abcd, horizontal_add(self.cache_cef, simd)),
            4 => self.cache_cef = simd,
            5 => self.cache_cef = horizontal_add(self.cache_cef, simd),
            6 => self.cache_g = simd,
            7 => self.cache_abcd = horizontal_add(
                self.cache_abcd,
                horizontal_add(
                    self.cache_cef,
                    horizontal_add(self.cache_g, simd)
                )
            ),
            _ => unreachable!(),
        }
        self.step += 1;
    }

    fn horizontal_add(a: Simd<u8, N>, b: Simd<u8, N>) -> Simd<u8, N> {
        a + b
    }

    pub fn finish(self) -> Simd<u8, N> {
        assert!(self.step == 8);
        self.cache_abcd
    }
}
