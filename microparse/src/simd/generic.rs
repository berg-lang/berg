use std::{ops::{Deref, DerefMut}, simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount}};

pub struct GenericMask<T: MaskElement, N: SupportedLaneCountFor<T>>(N::Mask<T>);

impl<T: MaskElement, N: SupportedLaneCountFor<T>> Deref for GenericMask<T, N> {
    type Target = N::Mask<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: MaskElement, N: SupportedLaneCountFor<T>> DerefMut for GenericMask<T, N> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

pub struct GenericSimd<T: SimdElement, N: GenericLaneCount>(N::Simd<T>);

impl<T: SimdElement, N: SupportedLaneCountFor<T::Mask>> Deref for GenericSimd<T, N> {
    type Target = N::Simd<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: SimdElement, N: SupportedLaneCountFor<T::Mask>> DerefMut for GenericSimd<T, N> {
    fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
        &mut self.0
    }
}

pub trait SupportedSimdWidth {
    const BITS: usize;
    type Lanes8:  SupportedLaneCountFor<i8,  SimdWidth = Self>;
    type Lanes16: SupportedLaneCountFor<i16, SimdWidth = Self>;
    type Lanes32: SupportedLaneCountFor<i32, SimdWidth = Self>;
    type Lanes64: SupportedLaneCountFor<i64, SimdWidth = Self>;
}
pub struct SimdWidth<const N: usize>;

pub trait GenericLaneCount: SupportedLaneCount {
    const LANES: usize;
    type Simd<T: SimdElement>;
    type Mask<T: MaskElement>;
}

impl<const N: usize> GenericLaneCount for LaneCount<N> where LaneCount<N>: SupportedLaneCount {
    const LANES: usize = N;
    type Simd<T: SimdElement> = Simd<T, N>;
    type Mask<T: MaskElement> = Mask<T, N>;
}

pub trait SupportedLaneCountFor<M: MaskElement>: GenericLaneCount {
    type SimdWidth: SupportedSimdWidth;
}

impl SupportedSimdWidth for SimdWidth<512> {
    const BITS: usize = 512;
    type Lanes8 = LaneCount<64>;
    type Lanes16 = LaneCount<32>;
    type Lanes32 = LaneCount<16>;
    type Lanes64 = LaneCount<8>;
}
impl SupportedLaneCountFor<i8>  for LaneCount<64> { type SimdWidth = SimdWidth<512>; }
impl SupportedLaneCountFor<i16> for LaneCount<32> { type SimdWidth = SimdWidth<512>; }
impl SupportedLaneCountFor<i32> for LaneCount<16> { type SimdWidth = SimdWidth<512>; }
impl SupportedLaneCountFor<i64> for LaneCount<8>  { type SimdWidth = SimdWidth<512>; }

impl SupportedSimdWidth for SimdWidth<256> {
    const BITS: usize = 256;
    type Lanes8 = LaneCount<32>;
    type Lanes16 = LaneCount<16>;
    type Lanes32 = LaneCount<8>;
    type Lanes64 = LaneCount<4>;
}
impl SupportedLaneCountFor<i8>  for LaneCount<32> { type SimdWidth = SimdWidth<256>; }
impl SupportedLaneCountFor<i16> for LaneCount<16> { type SimdWidth = SimdWidth<256>; }
impl SupportedLaneCountFor<i32> for LaneCount<8>  { type SimdWidth = SimdWidth<256>; }
impl SupportedLaneCountFor<i64> for LaneCount<4>  { type SimdWidth = SimdWidth<256>; }

impl SupportedSimdWidth for SimdWidth<128> {
    const BITS: usize = 128;
    type Lanes8 = LaneCount<16>;
    type Lanes16 = LaneCount<8>;
    type Lanes32 = LaneCount<4>;
    type Lanes64 = LaneCount<2>;
}
impl SupportedLaneCountFor<i8>  for LaneCount<16> { type SimdWidth = SimdWidth<128>; }
impl SupportedLaneCountFor<i16> for LaneCount<8>  { type SimdWidth = SimdWidth<128>; }
impl SupportedLaneCountFor<i32> for LaneCount<4>  { type SimdWidth = SimdWidth<128>; }
impl SupportedLaneCountFor<i64> for LaneCount<2>  { type SimdWidth = SimdWidth<128>; }

impl SupportedSimdWidth for SimdWidth<64> {
    const BITS: usize = 64;
    type Lanes8 = LaneCount<8>;
    type Lanes16 = LaneCount<4>;
    type Lanes32 = LaneCount<2>;
    type Lanes64 = LaneCount<1>;
}
impl SupportedLaneCountFor<i8>  for LaneCount<8>  { type SimdWidth = SimdWidth<64>; }
impl SupportedLaneCountFor<i16> for LaneCount<4>  { type SimdWidth = SimdWidth<64>; }
impl SupportedLaneCountFor<i32> for LaneCount<2>  { type SimdWidth = SimdWidth<64>; }
impl SupportedLaneCountFor<i64> for LaneCount<1>  { type SimdWidth = SimdWidth<64>; }

pub trait SupportedMask {
    type Elem: MaskElement;
    type LaneCount: SupportedLaneCountFor<Self::Elem>;
}
impl<E: MaskElement, N: SupportedLaneCountFor<E>> SupportedMask for GenericMask<E, N> {
    type Elem = E;
    type LaneCount = N;
}

pub trait SupportedSimd {
    type Elem: SimdElement;
    type LaneCount: SupportedLaneCountFor<<Self::Elem as SimdElement>::Mask>;
}

impl<E: SimdElement, N: SupportedLaneCountFor<E::Mask>> SupportedSimd for GenericSimd<E, N> {
    type Elem = E;
    type LaneCount = N;
}