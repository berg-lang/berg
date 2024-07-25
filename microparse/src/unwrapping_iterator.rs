///
/// An iterator or data structure with a known constant size at compile time, which can be converted
/// to an iterator which you don't need to unwrap as long as you only call next() for that known,
/// finite number of times.
/// 
pub trait IntoUnwrappingIterator<const N: usize> {
    type UnwrappingItem;
    type IntoUnwrappingIter: ExactSizeIterator<Item=Self::UnwrappingItem>;
    fn into_unwrapping_iter(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter>;
    #[allow(clippy::missing_safety_doc)]
    unsafe fn into_unwrapping_iter_unchecked(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter>;
}
impl<const N: usize, I: ConstSizeIterator<N>> IntoUnwrappingIterator<N> for I {
    type UnwrappingItem = I::Item;
    type IntoUnwrappingIter = Self;
    fn into_unwrapping_iter(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start(self)
    }
    unsafe fn into_unwrapping_iter_unchecked(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start_unchecked(self)
    }
}
impl<T, const N: usize> IntoUnwrappingIterator<N> for [T; N] {
    type UnwrappingItem = <Self as IntoIterator>::Item;
    type IntoUnwrappingIter = <Self as IntoIterator>::IntoIter;
    fn into_unwrapping_iter(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start(self.into_iter())
    }
    unsafe fn into_unwrapping_iter_unchecked(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start_unchecked(self.into_iter())
    }
}
impl<'a, T, const N: usize> IntoUnwrappingIterator<N> for &'a [T; N] {
    type UnwrappingItem = <Self as IntoIterator>::Item;
    type IntoUnwrappingIter = <Self as IntoIterator>::IntoIter;
    fn into_unwrapping_iter(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start(self.iter())
    }
    unsafe fn into_unwrapping_iter_unchecked(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start_unchecked(self.iter())
    }
}
impl<'a, T, const N: usize> IntoUnwrappingIterator<N> for &'a mut [T; N] {
    type UnwrappingItem = <Self as IntoIterator>::Item;
    type IntoUnwrappingIter = <Self as IntoIterator>::IntoIter;
    fn into_unwrapping_iter(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start(self.iter_mut())
    }
    unsafe fn into_unwrapping_iter_unchecked(self) -> UnwrappingIter<N, Self::IntoUnwrappingIter> {
        UnwrappingIter::start_unchecked(self.iter_mut())
    }
}

///
/// Existing iterator with a known constant size at compile time.
/// 
/// into_unwrapping_iter() is automatically implemented for these.
/// 
pub trait ConstSizeIterator<const N: usize>: ExactSizeIterator+Sized {}
impl<T, const N: usize> ConstSizeIterator<N> for std::array::IntoIter<T, N> {}
impl<T> ConstSizeIterator<0> for std::iter::Empty<T> {}
impl<T> ConstSizeIterator<1> for std::iter::Once<T> {}
impl<F: FnOnce()> ConstSizeIterator<1> for std::iter::OnceWith<F> {}

impl<const N: usize, I: ConstSizeIterator<N>, B, F: FnMut(I::Item) -> B> ConstSizeIterator<N> for std::iter::Map<I, F> {}
// TODO no const expressions
// impl<const LEN_A: usize, const LEN_B: usize, A: ConstSizeIterator<LEN_A>, B: ConstSizeIterator<LEN_B>> ConstSizeIterator<{ LEN_A+LEN_B }> for std::iter::Chain<A, B> {}
impl<'a, T: Clone+'a, const N: usize, I: ConstSizeIterator<N, Item=&'a T>> ConstSizeIterator<N> for std::iter::Cloned<I> {}
impl<'a, T: Copy+'a, const N: usize, I: ConstSizeIterator<N, Item=&'a T>> ConstSizeIterator<N> for std::iter::Copied<I> {}
impl<const N: usize, I: ConstSizeIterator<N>, F: FnMut(&I::Item)> ConstSizeIterator<N> for std::iter::Inspect<I, F> {}
impl<const N: usize, I: ConstSizeIterator<N>> ConstSizeIterator<N> for std::iter::Peekable<I> {}
impl<const N: usize, I: ConstSizeIterator<N>+DoubleEndedIterator> ConstSizeIterator<N> for std::iter::Rev<I> {}
// TODO not ExactSizeIterator, not clear why
// impl<const N: usize, I: ConstSizeIterator<N>, St, F> ConstSizeIterator<N> for std::iter::Scan<I, St, F> {}
// TODO no way to implement this correctly if they have different lengths
impl<const N: usize, A: ConstSizeIterator<N>, B: ConstSizeIterator<N>> ConstSizeIterator<N> for std::iter::Zip<A, B> {}
impl<const N: usize, I: ConstSizeIterator<N>> ConstSizeIterator<N> for Box<I> {}


///
/// Iterator that assumes you will call next() a known number of times.
/// 
/// # Panics
/// 
/// This number is checked in into_unwrapping_iter() and start(). next() will panic if you call it
/// more than the expected number of times. These panics should be elided in almost all cases where
/// you call into_unwrapping_iter(), as you know the . The only way for this to fail is if you have
/// already called next() on the iterator before creating an UnwrappingIter, which is what the panic
/// catches.
/// 
pub struct UnwrappingIter<const N: usize, I: ExactSizeIterator>(I);

impl<const N: usize, I: ExactSizeIterator> UnwrappingIter<N, I> {
    #[inline(always)]
    pub fn start(iter: I) -> UnwrappingIter<N, I> {
        if iter.len() != N {
            panic!("Expected iterator of length {}, got {}", N, iter.len());
        }
        UnwrappingIter(iter)
    }

    #[inline(always)]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn start_unchecked(iter: I) -> UnwrappingIter<N, I> {
        assert!(iter.len() == N, "Expected iterator of length {}, got {}", N, iter.len());
        UnwrappingIter(iter)
    }

    #[inline(always)]
    pub fn index(&self) -> usize {
        N - self.0.len()
    }

    #[inline(always)]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> I::Item {
        self.0.next().unwrap()
    }

    /// # Safety
    /// If you call next() more than N times after start(), this will assert.
    /// In general, this should be unnecessary: bounds checks should be optimized away on next()
    /// since the iterator has a known constant size at compile time.
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> I::Item {
        self.0.next().unwrap_unchecked()
    }
}

impl<I: ExactSizeIterator> UnwrappingIter<0, I> {
    pub fn fold<T>(self, init: T, _: impl FnMut(T, I::Item, usize) -> T) -> T {
        init
    }
}

macro_rules! impl_unwrapping_iter {
    ($n:literal: $($unchosen:literal)*) => {
        impl<I: ExactSizeIterator> UnwrappingIter<$n, I> {
            pub fn fold<T>(mut self, init: T, mut f: impl FnMut(T, I::Item, usize) -> T) -> T {
                let mut val = init;
                impl_unwrapping_iter! { @each f val self $($unchosen)* chose }
                val
            }
        }
    };
    (@each $f:ident $val:ident $self:ident $choice:literal $($unchosen:literal)* chose $($chosen:literal)*) => {
        impl_unwrapping_iter! { @each $f $val $self $($unchosen)* chose $($chosen)* $choice }
        impl_unwrapping_iter! { @each $f $val $self $($unchosen)* chose $($chosen)* }
    };
    (@each $f:ident $val:ident $self:ident chose $($chosen:literal)*) => {
        $val = $f($val, $self.next(), 0 $( + $chosen)*);
    }
}

impl<I: ExactSizeIterator> UnwrappingIter<1, I> {
    pub fn fold<T>(&mut self, init: T, mut f: impl FnMut(T, I::Item, usize) -> T) -> T {
        let val = init;
        f(val, self.next(), 0)
    }
}

impl_unwrapping_iter! {   2:                        1 }
impl_unwrapping_iter! {   4:                      2 1 }
impl_unwrapping_iter! {   8:                    4 2 1 }
impl_unwrapping_iter! {  16:                  8 4 2 1 }
impl_unwrapping_iter! {  32:               16 8 4 2 1 }
impl_unwrapping_iter! {  64:            32 16 8 4 2 1 }
impl_unwrapping_iter! { 128:         64 32 16 8 4 2 1 }
impl_unwrapping_iter! { 256:     128 64 32 16 8 4 2 1 }
impl_unwrapping_iter! { 512: 256 128 64 32 16 8 4 2 1 }
