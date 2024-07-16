use std::{cmp::Ordering, ops, simd::{cmp::*, LaneCount, Simd, SimdElement, SupportedLaneCount}, slice::ArrayChunks};
use crate::{AnyChunk, Chunk, ChunkOf, ChunkOrd, ChunkWithLanes};

use super::{AnyIntSimd, AnySignedSimd, AnySimd, SupportedSimd};

#[derive(Copy)]
#[repr(transparent)]
pub struct SimdChunk<T: SimdElement, const N: usize>(pub Simd<T, N>) where LaneCount<N>: SupportedLaneCount;

impl<T: SimdElement, const N: usize> AnyChunk for SimdChunk<T, N> where Simd<T, N>: AnySimd, LaneCount<N>: SupportedLaneCount {
    type Elem = T;
    type Mask = Mask<T>;
    const LANES: usize = N;

    #[inline]
    fn each_eq(&self, other: impl Into<Self>) -> Self::Mask {
        self.0.simd_eq(other.into().0).into()
    }
    #[inline]
    fn each_ne(&self, other: impl Into<Self>) -> Self::Mask {
        self.0.simd_ne(other.into().0).into()
    }
    #[inline]
    fn each_lt(&self, other: impl Into<Self>) -> Self::Mask {
        self.0.simd_lt(other.into().0).into()
    }
    #[inline]
    fn each_le(&self, other: impl Into<Self>) -> Self::Mask {
        self.0.simd_le(other.into().0).into()
    }
    #[inline]
    fn each_gt(&self, other: impl Into<Self>) -> Self::Mask {
        self.0.simd_gt(other.into().0).into()
    }
    #[inline]
    fn each_ge(&self, other: impl Into<Self>) -> Self::Mask {
        self.0.simd_ge(other.into().0).into()
    }
}

impl<T: SimdElement, const N: usize> ChunkOf<T> for SimdChunk<T, N> where Simd<T, N>: AnySimd, LaneCount<N>: SupportedLaneCount {}
impl<T: SimdElement, const N: usize> ChunkWithLanes<N> for SimdChunk<T, N> where Simd<T, N>: AnySimd, LaneCount<N>: SupportedLaneCount {}
impl<T: SimdElement, const N: usize> Chunk<T, N> for SimdChunk<T, N> where Simd<T, N>: AnySimd, LaneCount<N>: SupportedLaneCount {}

impl<T: SimdElement, const N: usize> ChunkOrd for SimdChunk<T, N> where Simd<T, N>: SimdOrd, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn each_max(&self, other: impl Into<Self>) -> Self {
        self.0.simd_max(other.into().0).into()
    }
    #[inline]
    fn each_min(&self, other: impl Into<Self>) -> Self {
        self.0.simd_min(other.into().0).into()
    }
    #[inline]
    fn each_clamp(&self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
        self.0.simd_clamp(min.into().0, max.into().0).into()
    }
}

//
// Conversions and
//
impl<T: SimdElement, const N: usize> From<Simd<T, N>> for SimdChunk<T, N> where LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn from(simd: Simd<T, N>) -> Self {
        Self(simd)
    }
}

impl<T: SimdElement, const N: usize> Into<Simd<T, N>> for SimdChunk<T, N> where LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn into(self) -> Simd<T, N> {
        self.0
    }
}

impl<T: SimdElement, const N: usize> From<[T; N]> for SimdChunk<T, N> where LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn from(array: [T; N]) -> Self {
        Self(Simd::from_array(array))
    }
}

impl<T: SimdElement, const N: usize> AsRef<Simd<T, N>> for SimdChunk<T, N> where LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn as_ref(&self) -> &Simd<T, N> {
        &self.0
    }
}

impl<T: SimdElement, const N: usize> AsMut<Simd<T, N>> for SimdChunk<T, N> where LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn as_mut(&mut self) -> &mut Simd<T, N> {
        &mut self.0
    }
}

//
// Implement core Rust proxies
//
impl<T: SimdElement, const N: usize> Clone for SimdChunk<T, N> where Simd<T, N>: Clone, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: SimdElement, const N: usize> Ord for SimdChunk<T, N> where Simd<T, N>: Ord, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>+Copy> PartialOrd<O> for SimdChunk<T, N> where Simd<T, N>: PartialOrd, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn partial_cmp(&self, other: &O) -> Option<Ordering> {
        self.0.partial_cmp(&(*other).into().0)
    }
}

impl<T: SimdElement, const N: usize> Eq for SimdChunk<T, N> where Simd<T, N>: Eq, LaneCount<N>: SupportedLaneCount {}

impl<T: SimdElement, const N: usize, O: Into<Self>+Copy> PartialEq<O> for SimdChunk<T, N> where Simd<T, N>: PartialEq<Simd<T, N>>, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.0.eq(&(*other).into().0)
    }
}

impl<T: SimdElement, const N: usize> ops::Not for SimdChunk<T, N> where Simd<T, N>: ops::Not<Output = Simd<T, N>>, LaneCount<N>: SupportedLaneCount {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        self.0.not().into()
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>> ops::BitAnd<O> for SimdChunk<T, N> where Simd<T, N>: ops::BitAnd<Output = Simd<T, N>>, LaneCount<N>: SupportedLaneCount {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: O) -> Self {
        self.0.bitand(rhs.into().0).into()
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>> ops::BitAndAssign<O> for SimdChunk<T, N> where Simd<T, N>: ops::BitAndAssign, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn bitand_assign(&mut self, rhs: O) {
        self.0.bitand_assign(rhs.into().0)
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>> ops::BitOr<O> for SimdChunk<T, N> where Simd<T, N>: ops::BitOr<Output = Simd<T, N>>, LaneCount<N>: SupportedLaneCount {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: O) -> Self {
        self.0.bitor(rhs.into().0).into()
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>> ops::BitOrAssign<O> for SimdChunk<T, N> where Simd<T, N>: ops::BitOrAssign, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn bitor_assign(&mut self, rhs: O) {
        self.0.bitor_assign(rhs.into().0)
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>> ops::BitXor<O> for SimdChunk<T, N> where Simd<T, N>: ops::BitXor<Output = Simd<T, N>>, LaneCount<N>: SupportedLaneCount {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: O) -> Self {
        self.0.bitxor(rhs.into().0).into()
    }
}

impl<T: SimdElement, const N: usize, O: Into<Self>> ops::BitXorAssign<O> for SimdChunk<T, N> where Simd<T, N>: ops::BitXorAssign, LaneCount<N>: SupportedLaneCount {
    #[inline]
    fn bitxor_assign(&mut self, rhs: O) {
        self.0.bitxor_assign(rhs.into().0)
    }
}

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
    pub fn for_each(self, mut f: impl FnMut(SimdChunk<T, N>)) {
        let remainder = self.0.remainder();
        for chunk in self.0 {
            f(From::from(Simd::from_array(*chunk)))
        }
        if !remainder.is_empty() {
            f(From::from(Simd::load_or(remainder, Simd::splat(self.1))));
        }
    }
}

struct SimdMask<const N: usize>(SimdMaskSize<N>::Primitive)

struct SimdMaskSize<const N: usize>();

trait SupportedSimdMaskSize<const N: usize> {
    type Primitive: Copy+Sized+PartialEq+PartialOrd;
}