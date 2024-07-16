use std::{
    ops,
    simd::{cmp::*, LaneCount, Simd, SimdElement, SupportedLaneCount}
};

mod private {
    pub trait Sealed {}
}

trait AnySimdBase: Sized+Copy+private::Sealed {}

pub trait AnySimd:
    Sized+Copy+private::Sealed
    +PartialOrd<Self>+PartialEq<Self>
    +ops::Add<Output=Self>+ops::Sub<Output=Self>+ops::Mul<Output=Self>+ops::Div<Output=Self>
    +SimdPartialEq+SimdPartialOrd
{
    type Elem: SimdElement+Sized;
    const LANES: usize;
    const BYTES: usize = Self::LANES * size_of::<Self::Elem>();
}

pub trait AnyIntSimd: AnySimd+ops::BitAnd<Output=Self>+ops::BitOr<Output=Self>+ops::BitXor<Output=Self>+ops::Not<Output=Self>+SimdOrd+Ord+Eq {}

pub trait AnySignedSimd: AnySimd+ops::Neg<Output=Self> {}

pub trait SimdForLanes<const N: usize>: AnySimd where LaneCount<N> : SupportedLaneCount {}

pub trait SupportedSimd<T: SimdElement, const N: usize>: SimdForLanes<N, Elem = T> where LaneCount<N>: SupportedLaneCount {}

pub trait AnySimdForSystemSize<const BYTES: usize> : AnySimd where LaneCount<BYTES>: SupportedLaneCount {}

pub trait SimdForSystemSize<T: SimdElement, const BYTES: usize> : AnySimdForSystemSize<BYTES>+AnySimd<Elem = T> where LaneCount<BYTES> : SupportedLaneCount {}

impl<T: SimdElement, const N: usize> private::Sealed for Simd<T, N> where
    LaneCount<N> : SupportedLaneCount {}

impl<T: SimdElement, const N: usize> AnySimdBase for Simd<T, N> where
    Self: Sized+Copy+private::Sealed,
    LaneCount<N> : SupportedLaneCount {}

impl<T: SimdElement, const N: usize> AnySimd for Simd<T, N> where
    Self: AnySimdBase
        +PartialOrd<Self>+PartialEq<Self>
        +ops::Add<Output=Self>+ops::Sub<Output=Self>+ops::Mul<Output=Self>+ops::Div<Output=Self>
        +SimdPartialEq+SimdPartialOrd,
    LaneCount<N> : SupportedLaneCount {
    type Elem = T;
    const LANES: usize = N;
}

impl<T: SimdElement, const N: usize> AnyIntSimd for Simd<T, N> where
    Self: AnySimd
        +ops::BitAnd<Output=Self>+ops::BitOr<Output=Self>+ops::BitXor<Output=Self>+ops::Not<Output=Self>
        +SimdOrd+Ord+Eq,
        LaneCount<N>: SupportedLaneCount {}

impl<T: SimdElement, const N: usize> AnySignedSimd for Simd<T, N> where
    Self: AnySimd+ops::Neg<Output=Self>,
    LaneCount<N>: SupportedLaneCount {}

impl<T: SimdElement, const N: usize> SimdForLanes<N> for Simd<T, N> where
    Self: AnySimd<Elem = T>,
    LaneCount<N> : SupportedLaneCount {}

impl<T: SimdElement, const N: usize> SupportedSimd<T, N> for Simd<T, N> where
    Self: SimdForLanes<N, Elem = T>,
    LaneCount<N> : SupportedLaneCount {}

macro_rules! impl_simd_system_sizes {
    (types: $t:tt; system_sizes: $e:tt;) => {
        impl_simd_system_sizes!{@elements types: $t; system_sizes: $e}
    };
    (@elements types: ($($t:ty),*); system_sizes: $e:tt) => {
        $(impl_simd_system_sizes!{@element_systems $t: $e})*
    };
    (@element_systems $t:ty: ($($bytes:literal),*)) => {
        $(
            impl AnySimdForSystemSize<$bytes> for Simd<$t, { $bytes / size_of::<$t>() }> {}
            impl SimdForSystemSize<$t, $bytes> for Simd<$t, { $bytes / size_of::<$t>() }> {}
        )*
    };
}

impl_simd_system_sizes! {
    types: (u8, i8);
    system_sizes: (1, 2, 4, 8, 16, 32, 64);
}
impl_simd_system_sizes! {
    types: (u16, i16);
    system_sizes: (2, 4, 8, 16, 32, 64);
}
impl_simd_system_sizes! {
    types: (u32, i32);
    system_sizes: (4, 8, 16, 32, 64);
}
impl_simd_system_sizes! {
    types: (u64, i64, f32, f64, usize, isize);
    system_sizes: (8, 16, 32, 64);
}

#[cfg(target_pointer_width = "16")]
impl_simd_system_sizes! {
    types: (usize, isize);
    system_sizes: (2, 4);
}
#[cfg(target_pointer_width = "32")]
impl_simd_system_sizes! {
    types: (usize, isize);
    system_sizes: (4);
}
