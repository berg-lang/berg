use std::{ops::{Deref, DerefMut}, simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount}};

//
// Mask/Simd
//

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct GenericMask<T: MaskElement, N: GenericLaneCount>(N::RawMask<T>);
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct GenericSimd<T: SimdElement, N: GenericLaneCount>(N::RawSimd<T>);
pub trait SupportedMask {
    type Elem: SupportedMaskElementFor<Self::SimdWidth, LaneCount = Self::LaneCount>;
    type LaneCount: SupportedLaneCountFor<Self::Elem, SimdWidth = Self::SimdWidth>;
    type SimdWidth: AnySimdWidth<Mask<Self::Elem>: SupportedMask<Elem = Self::Elem, LaneCount = Self::LaneCount>>;
}
pub trait SupportedSimd {
    type Elem: SimdElement<Mask: SupportedMaskElementFor<Self::SimdWidth, LaneCount = Self::LaneCount>>;
    type LaneCount: SupportedLaneCountFor<<Self::Elem as SimdElement>::Mask, SimdWidth = Self::SimdWidth>;
    type SimdWidth: AnySimdWidth<Mask<<Self::Elem as SimdElement>::Mask>: SupportedMask<Elem = <Self::Elem as SimdElement>::Mask, LaneCount = Self::LaneCount>>;
}

impl<T: MaskElement, N: SupportedLaneCountFor<T>> GenericMask<T, N> {
    pub fn new(mask: N::RawMask<T>) -> Self {
        Self(mask)
    }
    pub fn into_mask(self) -> N::RawMask<T> {
        self.0
    }
}
impl<T: SimdElement, N: SupportedLaneCountFor<T>> GenericSimd<T, N> {
    pub fn new(simd: N::RawSimd<T>) -> Self {
        Self(simd)
    }
    pub fn into_simd(self) -> N::RawSimd<T> {
        self.0
    }
}

impl<T: MaskElement, const N: usize> From<GenericMask<T, LaneCount<N>>> for Mask<T, N> where LaneCount<N>: SupportedLaneCountFor<T, RawMask<T>: Into<Mask<T, N>>> {
    fn from(from: GenericMask<T, LaneCount<N>>) -> Self {
        from.0.into()
    }
}
impl<T: SimdElement, const N: usize> From<GenericSimd<T, LaneCount<N>>> for Simd<T, N> where LaneCount<N>: SupportedLaneCountFor<T::Mask, RawSimd<T>: Into<Simd<T, N>>> {
    fn from(from: GenericSimd<T, LaneCount<N>>) -> Self {
        from.0.into()
    }
}
impl<T: MaskElement, const N: usize> From<Mask<T, N>> for GenericMask<T, LaneCount<N>> where LaneCount<N>: SupportedLaneCountFor<T, RawMask<T>: From<Mask<T, N>>> {
    fn from(from: Mask<T, N>) -> Self {
        Self(from.into())
    }
}
impl<T: SimdElement, const N: usize> From<Simd<T, N>> for GenericSimd<T, LaneCount<N>> where LaneCount<N>: SupportedLaneCountFor<T::Mask, RawSimd<T>: From<Simd<T, N>>> {
    fn from(from: Simd<T, N>) -> Self {
        Self(from.into())
    }
}

impl<E: SimdElement, N: SupportedLaneCountFor<E::Mask>> SupportedSimd for GenericSimd<E, N> where E::Mask: SupportedMaskElementFor<N::SimdWidth, LaneCount = N> {
    type Elem = E;
    type LaneCount = N;
    type SimdWidth = N::SimdWidth;
}

impl<E: MaskElement, N: SupportedLaneCountFor<E>> SupportedMask for GenericMask<E, N> where E: SupportedMaskElementFor<N::SimdWidth, LaneCount = N> {
    type Elem = E;
    type LaneCount = N;
    type SimdWidth = N::SimdWidth;
}

impl<T: MaskElement, N: SupportedLaneCountFor<T>> Deref for GenericMask<T, N> {
    type Target = N::RawMask<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: MaskElement, N: SupportedLaneCountFor<T>> DerefMut for GenericMask<T, N> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

impl<T: SimdElement, N: SupportedLaneCountFor<T::Mask>> Deref for GenericSimd<T, N> {
    type Target = N::RawSimd<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SimdElement, N: SupportedLaneCountFor<T::Mask>> DerefMut for GenericSimd<T, N> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

//
// LaneCount
//

pub trait GenericLaneCount: SupportedLaneCount {
    const LANES: usize;
    type RawSimd<T: SimdElement>: Copy;
    type RawMask<T: MaskElement>: Copy;
}

impl<const N: usize> GenericLaneCount for LaneCount<N> where LaneCount<N>: SupportedLaneCount {
    const LANES: usize = N;
    type RawSimd<T: SimdElement> = Simd<T, N>;
    type RawMask<T: MaskElement> = Mask<T, N>;
}

//
// SimdWidth
//

pub trait AnySimdWidth: Sized {
    const BITS: usize;
}
pub trait SupportedSimdWidth: SupportedSimdWidthFor<i8>+SupportedSimdWidthFor<i16>+SupportedSimdWidthFor<i32>+SupportedSimdWidthFor<i64> {
    type Mask64: GenericMaskElement;
    type Mask<T: GenericMaskElement>: SupportedMask<Elem = T>;
    type Simd<T: SimdElement<Mask: SupportedMaskElementFor<Self>>>: SupportedSimd<Elem = T, LaneCount = <T::Mask as SupportedMaskElementFor<Self>>::LaneCount>;
}
pub struct SimdWidth<const BITS: usize>;

impl<const BITS: usize> AnySimdWidth for SimdWidth<BITS> {
    const BITS: usize = BITS;
}
impl<const BITS: usize> SupportedSimdWidth for SimdWidth<BITS> {
    type Mask<T: SupportedMaskElementFor<Self>> = GenericMask<T, T::LaneCount>;
    type Simd<T: SimdElement<Mask: SupportedMaskElementFor<Self>>> = GenericSimd<T, <T::Mask as SupportedMaskElementFor<Self>>::LaneCount>;
}

//
// MaskElement / SimdElement
//

pub trait GenericSimdElement: SimdElement {}
pub trait GenericMaskElement: MaskElement {
    type LaneCountFor<W: AnySimdWidth>: SupportedLaneCountFor<Self, SimdWidth = W>;
    type SimdWidthFor<N: GenericLaneCount>: SupportedSimdWidthFor<Self, LaneCount = N>;
}
impl GenericMaskElement for i8 {
    type LaneCountFor<W: SupportedSimdWidthFor<Self>> = <Self as SupportedMaskElementFor<W>>::LaneCount;
    type SimdWidthFor<N: GenericLaneCount> = SimdWidth<N::LANES * size_of::<i8>() * 8>;
}
impl GenericSimdElement for i8 {}
impl GenericSimdElement for i16 {}
impl GenericSimdElement for i32 {}
impl GenericSimdElement for i64 {}
impl GenericSimdElement for u8 {}
impl GenericSimdElement for u16 {}
impl GenericSimdElement for u32 {}
impl GenericSimdElement for u64 {}
impl GenericSimdElement for f32 {}
impl GenericSimdElement for f64 {}
impl GenericSimdElement for isize {}
impl GenericSimdElement for usize {}

//
// Supported triples
//

pub trait SupportedMaskElementFor<W>: MaskElement {
    type LaneCount: SupportedLaneCountFor<Self, SimdWidth = W>;
}
pub trait SupportedLaneCountFor<M>: GenericLaneCount { // : SupportedMaskElementFor<Self::SimdWidth, LaneCount = Self>
    type SimdWidth: SupportedSimdWidthFor<M, LaneCount = Self>;
}
pub trait SupportedSimdWidthFor<M>: AnySimdWidth {
    type LaneCount: SupportedLaneCountFor<M, SimdWidth = Self>;
}

macro_rules! impl_simd_triples {
    ($($width:literal),*) => {
        $(impl_simd_triples! { $width = i8  * { $width / 8 / size_of::<i8>() } })*
        $(impl_simd_triples! { $width = i16 * { $width / 8 / size_of::<i16>() } })*
        $(impl_simd_triples! { $width = i32 * { $width / 8 / size_of::<i32>() } })*
        $(impl_simd_triples! { $width = i64 * { $width / 8 / size_of::<i64>() } })*
    };
    ($width:literal = $elem:ident * $lanes:expr) => {
        impl SupportedMaskElementFor<SimdWidth<$width>> for $elem { type LaneCount = LaneCount<$lanes>; }
        impl SupportedLaneCountFor<$elem> for LaneCount<$lanes> { type SimdWidth = SimdWidth<$width>; }
        impl SupportedSimdWidthFor<$elem> for SimdWidth<$width> { type LaneCount = LaneCount<$lanes>; }
    };
}
impl_simd_triples!(64, 128, 256, 512);

mod private {
    pub trait Sealed: Sized {}
}