use std::{ops, simd::SimdElement};

pub trait AnyChunk: Sized+PartialOrd<Self>+PartialEq<Self> {
    type Elem: SimdElement;
    type Mask: AnyMask; // TODO must be constrained to Mask<N>
    const LANES: usize;

    #[inline]
    fn each(&self) -> ChunkEach<Self> {
        ChunkEach(self)
    }

    fn each_eq(&self, other: impl Into<Self>) -> Self::Mask;
    fn each_ne(&self, other: impl Into<Self>) -> Self::Mask;
    fn each_lt(&self, other: impl Into<Self>) -> Self::Mask;
    fn each_le(&self, other: impl Into<Self>) -> Self::Mask;
    fn each_gt(&self, other: impl Into<Self>) -> Self::Mask;
    fn each_ge(&self, other: impl Into<Self>) -> Self::Mask;
}

pub trait ChunkOrd: AnyChunk {
    fn each_max(&self, other: impl Into<Self>) -> Self;
    fn each_min(&self, other: impl Into<Self>) -> Self;
    fn each_clamp(&self, min: impl Into<Self>, max: impl Into<Self>) -> Self;
}

pub trait ChunkOf<T: SimdElement> : AnyChunk<Elem = T> {}
pub trait ChunkWithLanes<const N: usize>: AnyChunk {}
pub trait Chunk<T: SimdElement, const N: usize> : ChunkOf<T> {}

pub trait AnyMask: Sized+Eq+PartialEq+From<bool>+ops::BitAnd+ops::BitOr+ops::BitXor+ops::Not {
    const LANES: usize;
    const ALL: Self;
    const NONE: Self;
    const EVENS: Self;
    const ODDS: Self;
}

pub trait Mask<const N: usize>: AnyMask {}

#[repr(transparent)]
pub struct PrimitiveMask<T: Sized+Copy>(pub T);

pub struct ChunkEach<'a, C: AnyChunk>(&'a C);

impl<'a, C: AnyChunk> ChunkEach<'a, C> {
    pub fn eq(&self, other: impl Into<C>) -> C::Mask {
        self.0.each_eq(other)
    }
    pub fn ne(&self, other: impl Into<C>) -> C::Mask {
        self.0.each_ne(other)
    }
    pub fn lt(&self, other: impl Into<C>) -> C::Mask {
        self.0.each_lt(other)
    }
    pub fn le(&self, other: impl Into<C>) -> C::Mask {
        self.0.each_le(other)
    }
    pub fn gt(&self, other: impl Into<C>) -> C::Mask {
        self.0.each_gt(other)
    }
    pub fn ge(&self, other: impl Into<C>) -> C::Mask {
        self.0.each_ge(other)
    }
}

impl<'a, C: ChunkOrd> ChunkEach<'a, C> {
    pub fn max(&self, other: impl Into<C>) -> C {
        self.0.each_max(other)
    }
    pub fn min(&self, other: impl Into<C>) -> C {
        self.0.each_min(other)
    }
    pub fn clamp(&self, min: impl Into<C>, max: impl Into<C>) -> C {
        self.0.each_clamp(min, max)
    }
}
