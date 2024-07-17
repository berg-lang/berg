use std::{simd::{LaneCount, Simd, SimdElement, SupportedLaneCount}, slice::ArrayChunks};

pub trait SimdChunksPadded<T: SimdElement> {
    ///
    /// Iterator that loads each chunk of a slice into a SIMD vector.
    ///
    /// If the last chunk is less than the size of the SIMD vector, it is padded with a default value
    /// passed in.
    ///
    fn simd_chunks_padded<const N: usize>(&self, default: T) -> SimdChunksPaddedIter<T, N> where LaneCount<N>: SupportedLaneCount;
}

impl<T: SimdElement, A: AsRef<[T]>> SimdChunksPadded<T> for A {
    #[inline]
    fn simd_chunks_padded<const N: usize>(&self, default: T) -> SimdChunksPaddedIter<T, N> where LaneCount<N>: SupportedLaneCount {
        SimdChunksPaddedIter::new(self.as_ref(), default)
    }
}

pub struct SimdChunksPaddedIter<'a, T: SimdElement, const N: usize>(ArrayChunks<'a, T, N>, T) where LaneCount<N>: SupportedLaneCount;

impl<'a, T: SimdElement, const N: usize> SimdChunksPaddedIter<'a, T, N> where LaneCount<N>: SupportedLaneCount {
    #[inline]
    pub fn new(slice: &'a [T], default: T) -> Self {
        Self(slice.array_chunks(), default)
    }

    #[inline]
    pub fn for_each(self, mut f: impl FnMut(Simd<T, N>)) {
        let remainder = self.0.remainder();
        for chunk in self.0 {
            f(Simd::from_array(*chunk))
        }
        if !remainder.is_empty() {
            f(Simd::load_or(remainder, Simd::splat(self.1)));
        }
    }
}

