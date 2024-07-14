use std::{simd::{cmp::*, Simd}, slice::ArrayChunks};
use derive_more::{From, Into};

use crate::byte_mask::ByteMask;

#[derive(Default, Debug, Clone, Copy, From, Into)]
#[repr(transparent)]
pub struct ByteChunk(pub Simd<u8, 64>);

impl From<u8> for ByteChunk {
    #[inline]
    fn from(value: u8) -> Self {
        ByteChunk(Simd::splat(value))
    }
}

impl ByteChunk {
    #[inline]
    pub fn simd_eq(&self, other: impl Into<Self>) -> ByteMask {
        self.0.simd_eq(other.into().0).into()
    }
    #[inline]
    pub fn simd_ne(&self, other: impl Into<Self>) -> ByteMask {
        self.0.simd_ne(other.into().0).into()
    }
    #[inline]
    pub fn simd_lt(&self, other: impl Into<Self>) -> ByteMask {
        self.0.simd_lt(other.into().0).into()
    }
    #[inline]
    pub fn simd_le(&self, other: impl Into<Self>) -> ByteMask {
        self.0.simd_le(other.into().0).into()
    }
    #[inline]
    pub fn simd_gt(&self, other: impl Into<Self>) -> ByteMask {
        self.0.simd_gt(other.into().0).into()
    }
    #[inline]
    pub fn simd_ge(&self, other: impl Into<Self>) -> ByteMask {
        self.0.simd_ge(other.into().0).into()
    }
    #[inline]
    pub fn simd_max(&self, other: impl Into<Self>) -> Self {
        self.0.simd_max(other.into().0).into()
    }
    #[inline]
    pub fn simd_min(&self, other: impl Into<Self>) -> Self {
        self.0.simd_min(other.into().0).into()
    }
    #[inline]
    pub fn simd_clamp(&self, min: impl Into<Self>, max: impl Into<Self>) -> Self {
        self.0.simd_clamp(min.into().0, max.into().0).into()
    }
}

pub trait BytesChunkPadded {
    ///
    /// Iterator that loads each chunk of a slice into a SIMD vector.
    ///
    /// If the last chunk is less than the size of the SIMD vector, it is padded with a default value
    /// passed in.
    ///
    fn byte_chunks_padded(&self, default: u8) -> ByteChunksPaddedIter;
}

impl<T: AsRef<[u8]>> BytesChunkPadded for T {
    #[inline]
    fn byte_chunks_padded(&self, default: u8) -> ByteChunksPaddedIter {
        ByteChunksPaddedIter::new(self.as_ref(), default)
    }
}

pub struct ByteChunksPaddedIter<'a>(ArrayChunks<'a, u8, 64>, u8);

impl<'a> ByteChunksPaddedIter<'a> {
    #[inline]
    pub fn new(slice: &'a [u8], default: u8) -> Self {
        Self(slice.array_chunks(), default)
    }

    #[inline]
    pub fn for_each(self, mut f: impl FnMut(ByteChunk)) {
        let remainder = self.0.remainder();
        for chunk in self.0 {
            f(Simd::from_array(*chunk).into())
        }
        if !remainder.is_empty() {
            f(Simd::load_or(remainder, Simd::splat(self.1)).into());
        }
    }
}

