use std::{ops::{Deref, DerefMut}, simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount}};

//
// Mask/Simd
//

pub struct GenericMask<T: MaskElement, N: GenericLaneCount>(N::RawMask<T>);
pub struct GenericSimd<T: SimdElement, N: GenericLaneCount>(N::RawSimd<T>);
pub trait SupportedMask {
    type Elem: SupportedMaskElementFor<Self::SimdWidth, LaneCount = Self::LaneCount>;
    type LaneCount: SupportedLaneCountFor<Self::Elem, SimdWidth = Self::SimdWidth>;
    type SimdWidth: SupportedSimdWidth<Mask<Self::Elem>: SupportedMask<Elem = Self::Elem, LaneCount = Self::LaneCount>>;
}
pub trait SupportedSimd {
    type Elem: SimdElement<Mask: SupportedMaskElementFor<Self::SimdWidth, LaneCount = Self::LaneCount>>;
    type LaneCount: SupportedLaneCountFor<<Self::Elem as SimdElement>::Mask, SimdWidth = Self::SimdWidth>;
    type SimdWidth: SupportedSimdWidth<Mask<<Self::Elem as SimdElement>::Mask>: SupportedMask<Elem = <Self::Elem as SimdElement>::Mask, LaneCount = Self::LaneCount>>;
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
    type RawSimd<T: SimdElement>;
    type RawMask<T: MaskElement>;
}

pub trait SupportedLaneCountFor<M>: GenericLaneCount { // : SupportedMaskElementFor<Self::SimdWidth, LaneCount = Self>
    type SimdWidth: SupportedSimdWidth;
}

impl<const N: usize> GenericLaneCount for LaneCount<N> where LaneCount<N>: SupportedLaneCount {
    const LANES: usize = N;
    type RawSimd<T: SimdElement> = Simd<T, N>;
    type RawMask<T: MaskElement> = Mask<T, N>;
}

//
// SimdWidth
//

pub trait SupportedSimdWidth: Sized {
    const BITS: usize;
    type Mask<T: SupportedMaskElementFor<Self>>: SupportedMask<Elem = T, LaneCount = T::LaneCount>;
    type Simd<T: SimdElement<Mask: SupportedMaskElementFor<Self>>>: SupportedSimd<Elem = T, LaneCount = <T::Mask as SupportedMaskElementFor<Self>>::LaneCount>;
}
pub struct SimdWidth<const BITS: usize>;

impl<const BITS: usize> SupportedSimdWidth for SimdWidth<BITS> where
    i8:  SupportedMaskElementFor<Self>,
    i16: SupportedMaskElementFor<Self>,
    i32: SupportedMaskElementFor<Self>,
    i64: SupportedMaskElementFor<Self>,
{
    const BITS: usize = BITS;
    type Mask<T: SupportedMaskElementFor<Self>> = GenericMask<T, T::LaneCount>;
    type Simd<T: SimdElement<Mask: SupportedMaskElementFor<Self>>> = GenericSimd<T, <T::Mask as SupportedMaskElementFor<Self>>::LaneCount>;
}

//
// MaskElement / SimdElement
//

pub trait SupportedMaskElementFor<W>: MaskElement {
    type LaneCount: SupportedLaneCountFor<Self, SimdWidth = W>;
}

//
// Supported triples
//

impl SupportedMaskElementFor<SimdWidth<512>> for i8  { type LaneCount = LaneCount<64>; }
impl SupportedMaskElementFor<SimdWidth<512>> for i16 { type LaneCount = LaneCount<32>; }
impl SupportedMaskElementFor<SimdWidth<512>> for i32 { type LaneCount = LaneCount<16>; }
impl SupportedMaskElementFor<SimdWidth<512>> for i64 { type LaneCount = LaneCount<8>; }
impl SupportedLaneCountFor<i8>  for LaneCount<64> { type SimdWidth = SimdWidth<512>; }
impl SupportedLaneCountFor<i16> for LaneCount<32> { type SimdWidth = SimdWidth<512>; }
impl SupportedLaneCountFor<i32> for LaneCount<16> { type SimdWidth = SimdWidth<512>; }
impl SupportedLaneCountFor<i64> for LaneCount<8>  { type SimdWidth = SimdWidth<512>; }

impl SupportedMaskElementFor<SimdWidth<256>> for i8  { type LaneCount = LaneCount<32>; }
impl SupportedMaskElementFor<SimdWidth<256>> for i16 { type LaneCount = LaneCount<16>; }
impl SupportedMaskElementFor<SimdWidth<256>> for i32 { type LaneCount = LaneCount<8>; }
impl SupportedMaskElementFor<SimdWidth<256>> for i64 { type LaneCount = LaneCount<4>; }
impl SupportedLaneCountFor<i8>  for LaneCount<32> { type SimdWidth = SimdWidth<256>; }
impl SupportedLaneCountFor<i16> for LaneCount<16> { type SimdWidth = SimdWidth<256>; }
impl SupportedLaneCountFor<i32> for LaneCount<8>  { type SimdWidth = SimdWidth<256>; }
impl SupportedLaneCountFor<i64> for LaneCount<4>  { type SimdWidth = SimdWidth<256>; }

impl SupportedMaskElementFor<SimdWidth<128>> for i8  { type LaneCount = LaneCount<16>; }
impl SupportedMaskElementFor<SimdWidth<128>> for i16 { type LaneCount = LaneCount<8>; }
impl SupportedMaskElementFor<SimdWidth<128>> for i32 { type LaneCount = LaneCount<4>; }
impl SupportedMaskElementFor<SimdWidth<128>> for i64 { type LaneCount = LaneCount<2>; }
impl SupportedLaneCountFor<i8>  for LaneCount<16> { type SimdWidth = SimdWidth<128>; }
impl SupportedLaneCountFor<i16> for LaneCount<8>  { type SimdWidth = SimdWidth<128>; }
impl SupportedLaneCountFor<i32> for LaneCount<4>  { type SimdWidth = SimdWidth<128>; }
impl SupportedLaneCountFor<i64> for LaneCount<2>  { type SimdWidth = SimdWidth<128>; }

impl SupportedMaskElementFor<SimdWidth<64>> for i8  { type LaneCount = LaneCount<8>; }
impl SupportedMaskElementFor<SimdWidth<64>> for i16 { type LaneCount = LaneCount<4>; }
impl SupportedMaskElementFor<SimdWidth<64>> for i32 { type LaneCount = LaneCount<2>; }
impl SupportedMaskElementFor<SimdWidth<64>> for i64 { type LaneCount = LaneCount<1>; }
impl SupportedLaneCountFor<i8>  for LaneCount<8> { type SimdWidth = SimdWidth<64>; }
impl SupportedLaneCountFor<i16> for LaneCount<4> { type SimdWidth = SimdWidth<64>; }
impl SupportedLaneCountFor<i32> for LaneCount<2> { type SimdWidth = SimdWidth<64>; }
impl SupportedLaneCountFor<i64> for LaneCount<1> { type SimdWidth = SimdWidth<64>; }
