use std::simd::{LaneCount, Mask, MaskElement, Simd, SimdElement, SupportedLaneCount};
use std::ops;

use generic_array::{typenum, ArrayLength, GenericArray};

pub trait ExposedLaneCount: SupportedLaneCount {
    const LANES: usize;
    type Simd<T: SimdElement>: AnySimd<Elem = T, LaneCount = Self>;
    type Mask<T: MaskElement>: AnyMask<Elem = T, LaneCount = Self>;
    type AsArrayLength: ArrayLength<Self::Simd<u8>>;
}

impl<const N: usize> ExposedLaneCount for LaneCount<N> where LaneCount<N>: SupportedLaneCount, typenum::Const<N>: typenum::ToUInt, typenum::U<N>: ArrayLength {
    const LANES: usize = N;
    type Simd<T: SimdElement> = Simd<T, N>;
    type Mask<T: MaskElement> = Mask<T, N>;
    type AsArrayLength = typenum::U<N>;
}

pub trait AnySimd: Sized {
    type Elem: SimdElement;
    type LaneCount: ExposedLaneCount;
    type Mask: AnyMask<Elem = <Self::Elem as SimdElement>::Mask, LaneCount = Self::LaneCount>;
    fn reverse(self) -> Self;
    fn rotate_elements_left<const OFFSET: usize>(self) -> Self;
    fn rotate_elements_right<const OFFSET: usize>(self) -> Self;
    fn interleave(self, other: Self) -> (Self, Self);
    fn deinterleave(self, other: Self) -> (Self, Self);
    fn resize<const M: usize>(self, value: Self::Elem) -> Simd<Self::Elem, M> where LaneCount<M>: SupportedLaneCount;
    /*const*/ fn len(&self) -> usize;
    fn splat(value: Self::Elem) -> Self;
    /*const*/ fn as_array(&self) -> &GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength>;
    fn as_mut_array(&mut self) -> &mut GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength>;
    /*const*/ fn from_array(array: GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength>) -> Self;
    /*const*/ fn to_array(self) -> GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength>;
    /*const*/ fn from_slice(slice: &[Self::Elem]) -> Self;
    fn copy_to_slice(self, slice: &mut [Self::Elem]);
    fn load_or_default(slice: &[Self::Elem]) -> Self where Self::Elem: Default;
    fn load_or(slice: &[Self::Elem], or: Self) -> Self;
    fn load_select_or_default(
        slice: &[Self::Elem],
        enable: Self::Mask
    ) -> Self where Self::Elem: Default;
    fn load_select(
        slice: &[Self::Elem],
        enable: Self::Mask,
        or: Self
    ) -> Self;
    unsafe fn load_select_unchecked(
        slice: &[Self::Elem],
        enable: Self::Mask,
        or: Self
    ) -> Self;
    unsafe fn load_select_ptr(
        ptr: *const Self::Elem,
        enable: Self::Mask,
        or: Self
    ) -> Self;
    fn gather_or(
        slice: &[Self::Elem],
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>,
        or: Self
    ) -> Self;
    fn gather_or_default(slice: &[Self::Elem], idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>) -> Self
    where
        Self::Elem: Default;
    fn gather_select(
        slice: &[Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>,
        or: Self
    ) -> Self;
    unsafe fn gather_select_unchecked(
        slice: &[Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>,
        or: Self
    ) -> Self;
    unsafe fn gather_ptr(source: <Self::LaneCount as ExposedLaneCount>::Simd<*const Self::Elem>) -> Self
    where
        Self::Elem: Default;
    unsafe fn gather_select_ptr(
        source: <Self::LaneCount as ExposedLaneCount>::Simd<*const Self::Elem>,
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        or: Self
    ) -> Self;
    fn store_select(
        self,
        slice: &mut [Self::Elem],
        enable: Self::Mask
    );
    unsafe fn store_select_unchecked(
        self,
        slice: &mut [Self::Elem],
        enable: Self::Mask
    );
    unsafe fn store_select_ptr(
        self,
        ptr: *mut Self::Elem,
        enable: Self::Mask
    );
    fn scatter(self, slice: &mut [Self::Elem], idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>);
    fn scatter_select(
        self,
        slice: &mut [Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>
    );
    unsafe fn scatter_select_unchecked(
        self,
        slice: &mut [Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>
    );
    unsafe fn scatter_ptr(self, dest: <Self::LaneCount as ExposedLaneCount>::Simd<*mut Self::Elem>);
    unsafe fn scatter_select_ptr(
        self,
        dest: <Self::LaneCount as ExposedLaneCount>::Simd<*mut Self::Elem>,
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>
    );
    // Only for T=u8
    // pub fn swizzle_dyn(self, idxs: Simd<u8, N>) -> Simd<u8, N>
}
impl<T: SimdElement, const N: usize> AnySimd for Simd<T, N> where LaneCount<N>: ExposedLaneCount {
    type Elem = T;
    type LaneCount = LaneCount<N>;
    type Mask = Mask<<T as SimdElement>::Mask, N>;
    #[inline(always)]
    fn reverse(self) -> Self { self.reverse() }
    #[inline(always)]
    fn rotate_elements_left<const OFFSET: usize>(self) -> Self { self.rotate_elements_left::<OFFSET>() }
    #[inline(always)]
    fn rotate_elements_right<const OFFSET: usize>(self) -> Self { self.rotate_elements_left::<OFFSET>()}
    #[inline(always)]
    fn interleave(self, other: Self) -> (Self, Self) { self.interleave(other) }
    #[inline(always)]
    fn deinterleave(self, other: Self) -> (Self, Self) { self.deinterleave(other) }
    #[inline(always)]
    fn resize<const M: usize>(self, value: Self::Elem) -> Simd<Self::Elem, M> where LaneCount<M>: SupportedLaneCount { self.resize(value) }
    #[inline(always)]
    /*const*/ fn len(&self) -> usize { self.len() }
    #[inline(always)]
    fn splat(value: Self::Elem) -> Self { Self::splat(value) }
    #[inline(always)]
    /*const*/ fn as_array(&self) -> &GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength> { self.as_array() }
    #[inline(always)]
    fn as_mut_array(&mut self) -> &mut GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength> { self.as_mut_array() }
    #[inline(always)]
    /*const*/ fn from_array(array: GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength>) -> Self { Self::from_array(array) }
    #[inline(always)]
    /*const*/ fn to_array(self) -> GenericArray<Self::Elem, <Self::LaneCount as ExposedLaneCount>::AsArrayLength> { self.to_array() }
    #[inline(always)]
    /*const*/ fn from_slice(slice: &[Self::Elem]) -> Self { Self::from_slice(slice) }
    #[inline(always)]
    fn copy_to_slice(self, slice: &mut [Self::Elem]) { self.copy_to_slice(slice) }
    #[inline(always)]
    fn load_or_default(slice: &[Self::Elem]) -> Self where Self::Elem: Default { Self::load_or_default(slice) }
    #[inline(always)]
    fn load_or(slice: &[Self::Elem], or: Self) -> Self { Self::load_or(slice, or) }
    #[inline(always)]
    fn load_select_or_default(
        slice: &[Self::Elem],
        enable: Self::Mask
    ) -> Self where Self::Elem: Default { Self::load_select_or_default(slice, enable) }
    #[inline(always)]
    fn load_select(
        slice: &[Self::Elem],
        enable: Self::Mask,
        or: Self
    ) -> Self { Self::load_select(slice, enable, or) }
    #[inline(always)]
    unsafe fn load_select_unchecked(
        slice: &[Self::Elem],
        enable: Self::Mask,
        or: Self
    ) -> Self { Self::load_select_unchecked(slice, enable, or) }
    #[inline(always)]
    unsafe fn load_select_ptr(
        ptr: *const Self::Elem,
        enable: Self::Mask,
        or: Self
    ) -> Self { Self::load_select_ptr(ptr, enable, or) }
    #[inline(always)]
    fn gather_or(
        slice: &[Self::Elem],
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>,
        or: Self
    ) -> Self { Self::gather_or(slice, idxs, or) }
    #[inline(always)]
    fn gather_or_default(slice: &[Self::Elem], idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>) -> Self
    where
        Self::Elem: Default { Self::gather_or_default(slice, idxs) }
    #[inline(always)]
    fn gather_select(
        slice: &[Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>,
        or: Self
    ) -> Self { Self::gather_select(slice, enable, idxs, or) }
    #[inline(always)]
    unsafe fn gather_select_unchecked(
        slice: &[Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>,
        or: Self
    ) -> Self { Self::gather_select_unchecked(slice, enable, idxs, or) }
    #[inline(always)]
    unsafe fn gather_ptr(source: <Self::LaneCount as ExposedLaneCount>::Simd<*const Self::Elem>) -> Self
    where
        Self::Elem: Default { Self::gather_ptr(source) }
    #[inline(always)]
    unsafe fn gather_select_ptr(
        source: <Self::LaneCount as ExposedLaneCount>::Simd<*const Self::Elem>,
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        or: Self
    ) -> Self { Self::gather_select_ptr(source, enable, or) }
    #[inline(always)]
    fn store_select(
        self,
        slice: &mut [Self::Elem],
        enable: Self::Mask
    ) { self.store_select(slice, enable) }
    #[inline(always)]
    unsafe fn store_select_unchecked(
        self,
        slice: &mut [Self::Elem],
        enable: Self::Mask
    ) { self.store_select_unchecked(slice, enable) }
    #[inline(always)]
    unsafe fn store_select_ptr(
        self,
        ptr: *mut Self::Elem,
        enable: Self::Mask
    ) { self.store_select_ptr(ptr, enable) }
    #[inline(always)]
    fn scatter(self, slice: &mut [Self::Elem], idxs: Simd<usize, N>) { self.scatter(slice, idxs) }
    #[inline(always)]
    fn scatter_select(
        self,
        slice: &mut [Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>
    ) { self.scatter_select(slice, enable, idxs) }
    #[inline(always)]
    unsafe fn scatter_select_unchecked(
        self,
        slice: &mut [Self::Elem],
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>,
        idxs: <Self::LaneCount as ExposedLaneCount>::Simd<usize>
    ) { self.scatter_select_unchecked(slice, enable, idxs) }
    #[inline(always)]
    unsafe fn scatter_ptr(self, dest: <Self::LaneCount as ExposedLaneCount>::Simd<*mut Self::Elem>) { self.scatter_ptr(dest) }
    #[inline(always)]
    unsafe fn scatter_select_ptr(
        self,
        dest: <Self::LaneCount as ExposedLaneCount>::Simd<*mut Self::Elem>,
        enable: <Self::LaneCount as ExposedLaneCount>::Mask<isize>
    ) { self.scatter_select_ptr(dest, enable) }
}

pub trait AnyMask:
        Sized+Clone+Default
        +ops::BitAnd<bool, Output = Self>+ops::BitAnd<Output = Self> {
    type Elem: MaskElement;
    type LaneCount: ExposedLaneCount;
    // type SimdWidth: SupportedSimdWidth;
    fn splat(value: bool) -> Self;
    // fn from_array(array: Self::Array) -> Self;
    // fn to_array(self) -> Self::Array;
    unsafe fn from_int_unchecked(value: <Self::LaneCount as ExposedLaneCount>::Simd<Self::Elem>) -> Self;
    fn from_int(value: <Self::LaneCount as ExposedLaneCount>::Simd<Self::Elem>) -> Self;
    fn to_int(self) -> <Self::LaneCount as ExposedLaneCount>::Simd<Self::Elem>;
    fn cast<U>(self) -> <Self::LaneCount as ExposedLaneCount>::Mask<U>
    where
        U: MaskElement;
    unsafe fn test_unchecked(&self, index: usize) -> bool;
    fn test(&self, index: usize) -> bool;
    unsafe fn set_unchecked(&mut self, index: usize, value: bool);
    fn set(&mut self, index: usize, value: bool);
    fn any(self) -> bool;
    fn all(self) -> bool;
    fn to_bitmask(self) -> u64;
    fn from_bitmask(bitmask: u64) -> Self;
    fn to_bitmask_vector(self) -> <Self::LaneCount as ExposedLaneCount>::Simd<u8>;
    fn from_bitmask_vector(bitmask: <Self::LaneCount as ExposedLaneCount>::Simd<u8>) -> Self;
    fn first_set(self) -> Option<usize>;
    // TODO haven't figured out why the impl fails
    // fn select<U>(
    //     self,
    //     true_values: <Self::LaneCount as ExposedLaneCount>::Simd<U>,
    //     false_values: <Self::LaneCount as ExposedLaneCount>::Simd<U>
    // ) -> <Self::LaneCount as ExposedLaneCount>::Simd<U>
    // where
    //     U: SupportedSimdElement<Mask = Self::Elem>;
    fn select_mask(
        self,
        true_values: Self,
        false_values: Self
    ) -> Self;
}
impl<T: MaskElement, const N: usize> AnyMask for Mask<T, N> where LaneCount<N>: SupportedLaneCount, Self: GetSimdWidth {
    type Elem = T;
    type LaneCount = LaneCount<N>;
    // type SimdWidth = <Self as GetSimdWidth>::SimdWidth;
    fn splat(value: bool) -> Self { Self::splat(value) }
    // fn from_array(array: Self::Array) -> Self { Self::from_array(array) }
    // fn to_array(self) -> Self::Array { self.to_array() }
    unsafe fn from_int_unchecked(value: <Self::LaneCount as ExposedLaneCount>::Simd<Self::Elem>) -> Self { Self::from_int_unchecked(value) }
    fn from_int(value: <Self::LaneCount as ExposedLaneCount>::Simd<Self::Elem>) -> Self { Self::from_int(value) }
    fn to_int(self) -> <Self::LaneCount as ExposedLaneCount>::Simd<Self::Elem> { self.to_int() }
    fn cast<U>(self) -> <Self::LaneCount as ExposedLaneCount>::Mask<U>
    where
        U: MaskElement { self.cast() }
    unsafe fn test_unchecked(&self, index: usize) -> bool { self.test_unchecked(index) }
    fn test(&self, index: usize) -> bool { self.test(index) }
    unsafe fn set_unchecked(&mut self, index: usize, value: bool) { self.set_unchecked(index, value) }
    fn set(&mut self, index: usize, value: bool) { self.set(index, value) }
    fn any(self) -> bool { self.any() }
    fn all(self) -> bool { self.all() }
    fn to_bitmask(self) -> u64 { self.to_bitmask() }
    fn from_bitmask(bitmask: u64) -> Self { Self::from_bitmask(bitmask) }
    fn to_bitmask_vector(self) -> <Self::LaneCount as ExposedLaneCount>::Simd<u8> { self.to_bitmask_vector() }
    fn from_bitmask_vector(bitmask: <Self::LaneCount as ExposedLaneCount>::Simd<u8>) -> Self { Self::from_bitmask_vector(bitmask) }
    fn first_set(self) -> Option<usize> { self.first_set() }
    // TODO haven't figured out why this fails
    // fn select<U>(
    //     self,
    //     true_values: <Self::LaneCount as ExposedLaneCount>::Simd<U>,
    //     false_values: <Self::LaneCount as ExposedLaneCount>::Simd<U>
    // ) -> <Self::LaneCount as ExposedLaneCount>::Simd<U>
    // where
    //     U: SimdElement<Mask = Self::Elem> { self.select::<U>(true_values, false_values) }
    fn select_mask(
        self,
        true_values: Self,
        false_values: Self
    ) -> Self { self.select_mask(true_values, false_values) }
}

pub trait SupportedMask: AnyMask+GetSimdWidth {
}

/// Get the element which is half the size of this one
trait DoubleMaskElement: MaskElement { type Double: MaskElement; }
impl DoubleMaskElement for i8 { type Double = i16; }
impl DoubleMaskElement for i16 { type Double = i32; }
impl DoubleMaskElement for i32 { type Double = i64; }
/// Get the element which is half the size of this one
trait HalfMaskElement: MaskElement { type Half: MaskElement; }
impl HalfMaskElement for i8 { type Half = i16; }
impl HalfMaskElement for i16 { type Half = i32; }
impl HalfMaskElement for i32 { type Half = i64; }

/// Get the mask which has a Double-sized element, but the same total width (double the lanes)
trait DoubleMask { type Double; }
impl<E: DoubleMaskElement> DoubleMask for Mask<E, 64> { type Double = Mask<E::Double, { 64 / 2 }>; }
impl<E: DoubleMaskElement> DoubleMask for Mask<E, 32> { type Double = Mask<E::Double, { 32 / 2 }>; }
impl<E: DoubleMaskElement> DoubleMask for Mask<E, 16> { type Double = Mask<E::Double, { 16 / 2 }>; }
impl<E: DoubleMaskElement> DoubleMask for Mask<E, 8> { type Double = Mask<E::Double, { 8 / 2 }>; }
impl<E: DoubleMaskElement> DoubleMask for Mask<E, 4> { type Double = Mask<E::Double, { 4 / 2 }>; }
impl<E: DoubleMaskElement> DoubleMask for Mask<E, 2> { type Double = Mask<E::Double, { 2 / 2 }>; }
/// Get the mask which has a Double-sized element, but the same total width (double the lanes)
trait HalfMask { type Half; }
impl<E: HalfMaskElement> HalfMask for Mask<E, 64> { type Half = Mask<E::Half, { 64 / 2 }>; }
impl<E: HalfMaskElement> HalfMask for Mask<E, 32> { type Half = Mask<E::Half, { 32 / 2 }>; }
impl<E: HalfMaskElement> HalfMask for Mask<E, 16> { type Half = Mask<E::Half, { 16 / 2 }>; }
impl<E: HalfMaskElement> HalfMask for Mask<E, 8> { type Half = Mask<E::Half, { 8 / 2 }>; }
impl<E: HalfMaskElement> HalfMask for Mask<E, 4> { type Half = Mask<E::Half, { 4 / 2 }>; }
impl<E: HalfMaskElement> HalfMask for Mask<E, 2> { type Half = Mask<E::Half, { 2 / 2 }>; }

trait GetSimdWidth { type SimdWidth: SupportedSimdWidth; }
impl<const N: usize> GetSimdWidth for Mask<i8, N> where LaneCount<N>: SupportedLaneCount, Self: SupportedSimdWidth { type SimdWidth = Self; }
impl<const N: usize> GetSimdWidth for Mask<i16, N> where LaneCount<N>: SupportedLaneCount, Self: HalfMask<Half: GetSimdWidth> { type SimdWidth = <<Self as HalfMask>::Half as GetSimdWidth>::SimdWidth; }
impl<const N: usize> GetSimdWidth for Mask<i32, N> where LaneCount<N>: SupportedLaneCount, Self: HalfMask<Half: GetSimdWidth> { type SimdWidth = <<Self as HalfMask>::Half as GetSimdWidth>::SimdWidth; }
impl<const N: usize> GetSimdWidth for Mask<i64, N> where LaneCount<N>: SupportedLaneCount, Self: HalfMask<Half: GetSimdWidth> { type SimdWidth = <<Self as HalfMask>::Half as GetSimdWidth>::SimdWidth; }

pub trait SupportedSimdWidth {
    type Mask8: AnyMask<Elem = i8>;
    type Mask16: AnyMask<Elem = i16>;
    type Mask32: AnyMask<Elem = i32>;
    type Mask64: AnyMask<Elem = i64>;
    type SimdU8: AnySimd<Elem = u8>;
    type SimdI8: AnySimd<Elem = i8>;
    type SimdU16: AnySimd<Elem = u16>;
    type SimdI16: AnySimd<Elem = i16>;
    type SimdU32: AnySimd<Elem = u32>;
    type SimdI32: AnySimd<Elem = i32>;
    type SimdU64: AnySimd<Elem = u64>;
    type SimdI64: AnySimd<Elem = i64>;
    type SimdF32: AnySimd<Elem = f32>;
    type SimdF64: AnySimd<Elem = f64>;
}
impl<const N: usize> SupportedSimdWidth for Mask<i8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        Self: AnyMask<Elem = i8>+DoubleMask<Double: AnyMask<Elem = i16>+DoubleMask<Double: AnyMask<Elem = i32>+DoubleMask<Double: AnyMask<Elem = i64>>>> {
    type Mask8 = Mask<i8, N>;
    type Mask16 = <Self::Mask8 as DoubleMask>::Double;
    type Mask32 = <Self::Mask16 as DoubleMask>::Double;
    type Mask64 = <Self::Mask32 as DoubleMask>::Double;
    type SimdU8 = <<Self::Mask8 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<u8>;
    type SimdI8 = <<Self::Mask8 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<i8>;
    type SimdU16 = <<Self::Mask16 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<u16>;
    type SimdI16 = <<Self::Mask16 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<i16>;
    type SimdU32 = <<Self::Mask32 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<u32>;
    type SimdI32 = <<Self::Mask32 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<i32>;
    type SimdU64 = <<Self::Mask64 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<u64>;
    type SimdI64 = <<Self::Mask64 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<i64>;
    type SimdF32 = <<Self::Mask32 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<f32>;
    type SimdF64 = <<Self::Mask64 as AnyMask>::LaneCount as ExposedLaneCount>::Simd<f64>;
}

pub trait GenericSimdElement: SimdElement+Copy+Clone {
    type Mask: MaskElement;
    type Simd<const N: usize>: AnySimd<Elem = Self, LaneCount = LaneCount<N>> where LaneCount<N>: SupportedLaneCount;
}
impl<T: SimdElement<Mask: MaskElement>+Copy+Clone> GenericSimdElement for T {
    type Mask = T::Mask;
    type Simd<const N: usize> = Simd<Self, N> where LaneCount<N>: SupportedLaneCount;
}
pub trait GenericMaskElement: GenericSimdElement<Mask=Self>+MaskElement {
}
impl<T: MaskElement+Copy+Clone> GenericMaskElement for T {
}