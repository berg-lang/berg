use crate::IntoRange;
use std::borrow::{Borrow, BorrowMut, Cow};
use std::cmp::Ordering;
use std::fmt;
use std::iter::*;
use std::marker::PhantomData;
use std::ops::{
    Add, AddAssign, Bound, Deref, DerefMut, Index, IndexMut, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive, Sub, SubAssign
};
use std::slice::{Iter, IterMut, SliceIndex};

// index_type and util::indexed_vec work together to let you use a custom type
// (like TokenIndex) to index the vector, and disallow any other type (like usize
// or SourceIndex) from accessing it, so that you can be sure you are accessing
// the thing you think you are!
//
// Think of index_type (TokenIndex/SourceIndex/ByteIndex) as your number and
// the IndexedVec as Vec<Token>/Vec<SourceSpec>, and you will be good.

#[macro_export]
macro_rules! index_type {
    ($(pub struct $name:ident(pub $($type:tt)*) $(with $($trait:tt),*)* <= $max:expr ;)*) => {
        use $crate::{Delta,IndexType};
        use std::fmt;
        use std::ops::{Add,AddAssign,Sub,SubAssign};
        use std::cmp::Ordering;
        $(
            #[derive(Copy,Clone,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]
            #[repr(transparent)]
            pub struct $name(pub $($type)*);
            impl PartialEq<usize> for $name {
                fn eq(&self, other: &usize) -> bool { (self.0 as usize).eq(other) }
            }
            $($(index_type! { @trait $name $trait })*)*
            impl PartialOrd<usize> for $name {
                fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
                    (self.0 as usize).partial_cmp(other)
                }
                fn lt(&self, other: &usize) -> bool { (self.0 as usize).lt(other) }
                fn le(&self, other: &usize) -> bool { (self.0 as usize).le(other) }
                fn gt(&self, other: &usize) -> bool { (self.0 as usize).gt(other) }
                fn ge(&self, other: &usize) -> bool { (self.0 as usize).ge(other) }
            }
            impl IndexType for $name { }
            impl $name { pub const MAX: $name = $name($max); }
            impl From<usize> for $name { fn from(size: usize) -> Self { $name(size as $($type)*) } }
            impl From<$name> for usize { fn from(size: $name) -> Self { size.0 as usize } }
            impl Add<usize> for $name { type Output = Self; fn add(self, value: usize) -> Self { $name(self.0 + Self::from(value).0) } }
            impl Add<Delta<Self>> for $name { type Output = Self; fn add(self, value: Delta<Self>) -> Self { $name(self.0 + (value.0).0) } }
            impl Sub<usize> for $name { type Output = Self; fn sub(self, value: usize) -> Self { $name(self.0 - Self::from(value).0) } }
            impl Sub<Delta<Self>> for $name { type Output = Self; fn sub(self, value: Delta<Self>) -> Self { $name(self.0 - (value.0).0) } }
            impl Sub<Self> for $name { type Output = Delta<Self>; fn sub(self, value: Self) -> Delta<Self> { Delta(self - (value.0 as usize)) } }
            impl AddAssign<usize> for $name { fn add_assign(&mut self, value: usize) { *self = *self + value } }
            impl AddAssign<Delta<Self>> for $name { fn add_assign(&mut self, value: Delta<Self>) { *self = *self + value } }
            impl SubAssign<usize> for $name { fn sub_assign(&mut self, value: usize) { *self = *self - value } }
            impl SubAssign<Delta<Self>> for $name { fn sub_assign(&mut self, value: Delta<Self>) { *self = *self - value } }
        )*
    };
    (@trait $name:ident Display) => {
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    (@trait $name:ident Debug) => {
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Delta<IndexType>(pub IndexType);
impl<T: IndexType> fmt::Display for Delta<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: IndexType> From<Delta<T>> for usize {
    fn from(size: Delta<T>) -> Self {
        size.0.into()
    }
}
impl<T: IndexType> From<usize> for Delta<T> {
    fn from(size: usize) -> Self {
        Delta(T::from(size))
    }
}
impl<T: IndexType> PartialEq<usize> for Delta<T> {
    fn eq(&self, other: &usize) -> bool {
        self.0.eq(other)
    }
}
impl<T: IndexType> PartialOrd<usize> for Delta<T> {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
    fn lt(&self, other: &usize) -> bool {
        self.0 < *other
    }
    fn le(&self, other: &usize) -> bool {
        self.0 <= *other
    }
    fn gt(&self, other: &usize) -> bool {
        self.0 > *other
    }
    fn ge(&self, other: &usize) -> bool {
        self.0 >= *other
    }
}

impl<T: IndexType> Add<usize> for Delta<T> {
    type Output = Self;
    fn add(self, value: usize) -> Self {
        Delta(self.0 + value)
    }
}
impl<T: IndexType> Add for Delta<T> {
    type Output = Self;
    fn add(self, value: Self) -> Self {
        Delta(self.0 + value)
    }
}
impl<T: IndexType> Sub<usize> for Delta<T> {
    type Output = Self;
    fn sub(self, value: usize) -> Self {
        Delta(self.0 - value)
    }
}
impl<T: IndexType> Sub for Delta<T> {
    type Output = Self;
    fn sub(self, value: Self) -> Self {
        Delta(self.0 - value)
    }
}
impl<T: IndexType> AddAssign<usize> for Delta<T> {
    fn add_assign(&mut self, value: usize) {
        *self = *self + value
    }
}
impl<T: IndexType> AddAssign for Delta<T> {
    fn add_assign(&mut self, value: Self) {
        *self = *self + value
    }
}
impl<T: IndexType> SubAssign<usize> for Delta<T> {
    fn sub_assign(&mut self, value: usize) {
        *self = *self - value
    }
}
impl<T: IndexType> SubAssign for Delta<T> {
    fn sub_assign(&mut self, value: Self) {
        *self = *self - value
    }
}

pub trait IndexType:
    Copy
    + Clone
    + fmt::Display
    + Into<usize>
    + From<usize>
    + PartialOrd
    + PartialEq
    + PartialOrd<usize>
    + PartialEq<usize>
    + Sub<Self, Output = Delta<Self>>
    + AddAssign<usize>
    + SubAssign<usize>
    + AddAssign<Delta<Self>>
    + SubAssign<Delta<Self>>
    + Add<usize, Output = Self>
    + Sub<usize, Output = Self>
    + Add<Delta<Self>, Output = Self>
    + Sub<Delta<Self>, Output = Self>
{
    fn into_slice_index(self) -> usize {
        self.into()
    }
}

pub struct IndexedIter<Inner: Iterator, Idx: IndexType>(Inner, PhantomData<Idx>);
pub struct EnumerateIndex<Inner: Iterator, Idx: IndexType>(Inner, Idx, PhantomData<Idx>);

impl<Inner: Iterator, Idx: IndexType> From<Inner> for IndexedIter<Inner, Idx> {
    fn from(inner: Inner) -> Self {
        IndexedIter(inner, PhantomData)
    }
}

impl<Inner: Iterator, Idx: IndexType> IndexedIter<Inner, Idx> {
    // Functions where the indices just don't matter
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Inner::Item> {
        self.0.next()
    }
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    pub fn count(self) -> usize {
        self.0.count()
    }
    pub fn last(self) -> Option<Inner::Item> {
        self.0.last()
    }
    pub fn nth(&mut self, n: usize) -> Option<Inner::Item> {
        self.0.nth(n)
    }
    pub fn by_ref(&mut self) -> &mut Self {
        self
    }
    pub fn for_each<F>(self, f: F)
    where
        F: FnMut(Inner::Item),
    {
        self.0.for_each(f)
    }
    pub fn find<P>(&mut self, predicate: P) -> Option<Inner::Item>
    where
        P: FnMut(&Inner::Item) -> bool,
    {
        self.0.find(predicate)
    }
    pub fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Inner::Item) -> B,
    {
        self.0.fold(init, f)
    }
    pub fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Inner::Item) -> bool,
    {
        self.0.all(f)
    }
    pub fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Inner::Item) -> bool,
    {
        self.0.any(f)
    }
    pub fn max(self) -> Option<Inner::Item>
    where
        Inner::Item: Ord,
    {
        self.0.max()
    }
    pub fn min(self) -> Option<Inner::Item>
    where
        Inner::Item: Ord,
    {
        self.0.min()
    }
    pub fn max_by_key<B, F>(self, f: F) -> Option<Inner::Item>
    where
        B: Ord,
        F: FnMut(&Inner::Item) -> B,
    {
        self.0.max_by_key(f)
    }
    pub fn max_by<F>(self, compare: F) -> Option<Inner::Item>
    where
        F: FnMut(&Inner::Item, &Inner::Item) -> Ordering,
    {
        self.0.max_by(compare)
    }
    pub fn min_by_key<B, F>(self, f: F) -> Option<Inner::Item>
    where
        B: Ord,
        F: FnMut(&Inner::Item) -> B,
    {
        self.0.min_by_key(f)
    }
    pub fn min_by<F>(self, compare: F) -> Option<Inner::Item>
    where
        F: FnMut(&Inner::Item, &Inner::Item) -> Ordering,
    {
        self.0.min_by(compare)
    }
    pub fn sum<S>(self) -> S
    where
        S: Sum<Inner::Item>,
    {
        self.0.sum()
    }
    pub fn product<P>(self) -> P
    where
        P: Product<Inner::Item>,
    {
        self.0.product()
    }
    pub fn cmp<I>(self, other: I) -> Ordering
    where
        I: IntoIterator<Item = Inner::Item>,
        Inner::Item: Ord,
    {
        self.0.cmp(other)
    }
    pub fn partial_cmp<I>(self, other: I) -> Option<Ordering>
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.partial_cmp(other)
    }
    pub fn eq<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialEq<<I as IntoIterator>::Item>,
    {
        self.0.eq(other)
    }
    pub fn ne<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialEq<<I as IntoIterator>::Item>,
    {
        self.0.ne(other)
    }
    pub fn lt<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.lt(other)
    }
    pub fn le<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.le(other)
    }
    pub fn gt<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.gt(other)
    }
    pub fn ge<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.ge(other)
    }

    // Functions where indices are no longer valid:
    pub fn partition<B, F>(self, f: F) -> (B, B)
    where
        B: Default + Extend<Inner::Item>,
        F: FnMut(&Inner::Item) -> bool,
    {
        self.0.partition(f)
    }
    pub fn chain<U>(self, other: U) -> Chain<Inner, <U as IntoIterator>::IntoIter>
    where
        U: IntoIterator<Item = Inner::Item>,
    {
        self.0.chain(other)
    }
    pub fn filter<P>(self, predicate: P) -> Filter<Inner, P>
    where
        P: FnMut(&Inner::Item) -> bool,
    {
        self.0.filter(predicate)
    }
    pub fn filter_map<B, F>(self, f: F) -> FilterMap<Inner, F>
    where
        F: FnMut(Inner::Item) -> Option<B>,
    {
        self.0.filter_map(f)
    }
    pub fn skip_while<P>(self, predicate: P) -> SkipWhile<Inner, P>
    where
        P: FnMut(&Inner::Item) -> bool,
    {
        self.0.skip_while(predicate)
    }
    pub fn skip(self, n: usize) -> Skip<Inner> {
        self.0.skip(n)
    }
    pub fn flat_map<U, F>(self, f: F) -> FlatMap<Inner, U, F>
    where
        F: FnMut(Inner::Item) -> U,
        U: IntoIterator,
    {
        self.0.flat_map(f)
    }
    pub fn rev(self) -> Rev<Inner>
    where
        Inner: DoubleEndedIterator,
    {
        self.0.rev()
    }
    pub fn cycle(self) -> Cycle<Inner>
    where
        Inner: Clone,
    {
        self.0.cycle()
    }

    // Functions that yield usize (and that we want to return Idx)--the whole point of this:
    pub fn enumerate(self) -> IndexedIter<EnumerateIndex<Inner, Idx>, Idx> {
        EnumerateIndex(self.0, 0.into(), PhantomData).into()
    }
    pub fn position<P>(&mut self, predicate: P) -> Option<Idx>
    where
        P: FnMut(Inner::Item) -> bool,
    {
        self.0.position(predicate).map(Into::into)
    }
    pub fn rposition<P>(&mut self, predicate: P) -> Option<Idx>
    where
        P: FnMut(Inner::Item) -> bool,
        Self: ExactSizeIterator + DoubleEndedIterator,
        Inner: ExactSizeIterator + DoubleEndedIterator,
    {
        self.0.rposition(predicate).map(Into::into)
    }

    // Functions we want to keep the index for (make it still possible to get valid indices):
    pub fn zip<U>(self, other: U) -> IndexedIter<Zip<Inner, <U as IntoIterator>::IntoIter>, Idx>
    where
        U: IntoIterator,
    {
        self.0.zip(other).into()
    }
    pub fn map<B, F>(self, f: F) -> IndexedIter<Map<Inner, F>, Idx>
    where
        F: FnMut(Inner::Item) -> B,
    {
        self.0.map(f).into()
    }
    pub fn take_while<P>(self, predicate: P) -> IndexedIter<TakeWhile<Inner, P>, Idx>
    where
        P: FnMut(&Inner::Item) -> bool,
    {
        self.0.take_while(predicate).into()
    }
    pub fn take(self, n: usize) -> IndexedIter<Take<Inner>, Idx> {
        self.0.take(n).into()
    }
    pub fn scan<St, B, F>(self, initial_state: St, f: F) -> IndexedIter<Scan<Inner, St, F>, Idx>
    where
        F: FnMut(&mut St, Inner::Item) -> Option<B>,
    {
        self.0.scan(initial_state, f).into()
    }
    pub fn fuse(self) -> IndexedIter<Fuse<Inner>, Idx> {
        self.0.fuse().into()
    }
    pub fn inspect<F>(self, f: F) -> IndexedIter<Inspect<Inner, F>, Idx>
    where
        F: FnMut(&Inner::Item),
    {
        self.0.inspect(f).into()
    }
    pub fn cloned<'a, T>(self) -> IndexedIter<Cloned<Inner>, Idx>
    where
        Inner: Iterator<Item = &'a T>,
        T: 'a + Clone,
    {
        self.0.cloned().into()
    }

    // Functions we would've liked to keep the index for, but we don't feel like implementing right now:
    pub fn peekable(self) -> Peekable<Inner> {
        self.0.peekable()
    }
    pub fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB)
    where
        FromA: Default + Extend<A>,
        FromB: Default + Extend<B>,
        Inner: Iterator<Item = (A, B)>,
    {
        self.0.unzip()
    }
    pub fn collect<B>(self) -> B
    where
        B: FromIterator<Inner::Item>,
    {
        self.0.collect()
    }
}

impl<Inner: Iterator, Idx: IndexType> Iterator for IndexedIter<Inner, Idx> {
    type Item = Inner::Item;
    // Functions where the indices just don't matter
    fn next(&mut self) -> Option<Inner::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize {
        self.0.count()
    }
    fn last(self) -> Option<Inner::Item> {
        self.0.last()
    }
    fn nth(&mut self, n: usize) -> Option<Inner::Item> {
        self.0.nth(n)
    }
    fn by_ref(&mut self) -> &mut Self {
        self
    }
    fn for_each<F>(self, f: F)
    where
        F: FnMut(Inner::Item),
    {
        self.0.for_each(f)
    }
    fn find<P>(&mut self, predicate: P) -> Option<Inner::Item>
    where
        P: FnMut(&Inner::Item) -> bool,
    {
        self.0.find(predicate)
    }
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, Inner::Item) -> B,
    {
        self.0.fold(init, f)
    }
    fn all<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Inner::Item) -> bool,
    {
        self.0.all(f)
    }
    fn any<F>(&mut self, f: F) -> bool
    where
        F: FnMut(Inner::Item) -> bool,
    {
        self.0.any(f)
    }
    fn max(self) -> Option<Inner::Item>
    where
        Inner::Item: Ord,
    {
        self.0.max()
    }
    fn min(self) -> Option<Inner::Item>
    where
        Inner::Item: Ord,
    {
        self.0.min()
    }
    fn max_by_key<B, F>(self, f: F) -> Option<Inner::Item>
    where
        B: Ord,
        F: FnMut(&Inner::Item) -> B,
    {
        self.0.max_by_key(f)
    }
    fn max_by<F>(self, compare: F) -> Option<Inner::Item>
    where
        F: FnMut(&Inner::Item, &Inner::Item) -> Ordering,
    {
        self.0.max_by(compare)
    }
    fn min_by_key<B, F>(self, f: F) -> Option<Inner::Item>
    where
        B: Ord,
        F: FnMut(&Inner::Item) -> B,
    {
        self.0.min_by_key(f)
    }
    fn min_by<F>(self, compare: F) -> Option<Inner::Item>
    where
        F: FnMut(&Inner::Item, &Inner::Item) -> Ordering,
    {
        self.0.min_by(compare)
    }
    fn sum<S>(self) -> S
    where
        S: Sum<Inner::Item>,
    {
        self.0.sum()
    }
    fn product<P>(self) -> P
    where
        P: Product<Inner::Item>,
    {
        self.0.product()
    }
    fn cmp<I>(self, other: I) -> Ordering
    where
        I: IntoIterator<Item = Inner::Item>,
        Inner::Item: Ord,
    {
        self.0.cmp(other)
    }
    fn partial_cmp<I>(self, other: I) -> Option<Ordering>
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.partial_cmp(other)
    }
    fn eq<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialEq<<I as IntoIterator>::Item>,
    {
        self.0.eq(other)
    }
    fn ne<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialEq<<I as IntoIterator>::Item>,
    {
        self.0.ne(other)
    }
    fn lt<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.lt(other)
    }
    fn le<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.le(other)
    }
    fn gt<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.gt(other)
    }
    fn ge<I>(self, other: I) -> bool
    where
        I: IntoIterator,
        Inner::Item: PartialOrd<<I as IntoIterator>::Item>,
    {
        self.0.ge(other)
    }

    // Functions where indices are no longer valid:
    fn partition<B, F>(self, f: F) -> (B, B)
    where
        B: Default + Extend<Inner::Item>,
        F: FnMut(&Inner::Item) -> bool,
    {
        self.0.partition(f)
    }
    // fn chain<U>(self, other: U) -> Chain<Inner, <U as IntoIterator>::IntoIter> where U: IntoIterator<Item = Inner::Item> { self.0.chain(other) }
    // fn filter<P>(self, predicate: P) -> Filter<Inner, P> where P: FnMut(&Inner::Item) -> bool { self.0.filter(predicate) }
    // fn filter_map<B, F>(self, f: F) -> FilterMap<Inner, F> where F: FnMut(Inner::Item) -> Option<B> { self.0.filter_map(f) }
    // fn skip_while<P>(self, predicate: P) -> SkipWhile<Inner, P> where P: FnMut(&Inner::Item) -> bool { self.0.skip_while(predicate) }
    // fn skip(self, n: usize) -> Skip<Inner> { self.0.skip(n) }
    // fn flat_map<U, F>(self, f: F) -> FlatMap<Inner, U, F> where F: FnMut(Inner::Item) -> U, U: IntoIterator { self.0.flat_map(f) }
    // fn rev(self) -> Rev<Inner> where Inner: DoubleEndedIterator { self.0.rev() }
    // fn cycle(self) -> Cycle<Inner> where Inner: Clone { self.0.cycle() }

    // Functions that yield usize (and that we want to return Idx)--the whole point of this:
    // fn enumerate(self) -> Enumerate<Self> { }
    fn position<P>(&mut self, predicate: P) -> Option<usize>
    where
        P: FnMut(Inner::Item) -> bool,
    {
        self.0.position(predicate)
    }

    // Functions we want to keep the index for (make it still possible to get valid indices):
    // fn zip<U>(self, other: U) -> Zip<Self, <U as IntoIterator>::IntoIter> where U: IntoIterator { self.0.zip(other) }
    // fn map<B, F>(self, f: F) -> Map<Self, F> where F: FnMut(Self::Item) -> B { self.0.map(f) }
    // fn take_while<P>(self, predicate: P) -> TakeWhile<Self, P> where P: FnMut(&Self::Item) -> bool { self.0.take_while(predicate) }
    // fn take(self, n: usize) -> Take<Self>,Idx> { self.0.take(n) }
    // fn scan<St, B, F>(self, initial_state: St, f: F) -> Scan<Inner, St, F> where F: FnMut(&mut St, Inner::Item) -> Option<B> { self.0.scan(initial_state, f).into() }
    // fn fuse(self) -> Fuse<Inner> { self.0.fuse().into() }
    // fn inspect<F>(self, f: F) -> Inspect<Inner, F> where F: FnMut(&Inner::Item) -> () { self.0.inspect(f).into() }
    // fn cloned<'a, T>(self) -> Cloned<Inner> where Inner:Iterator<Item=&'a T>, T: 'a + Clone { self.0.cloned().into() }

    // Functions we would've liked to keep the index for, but we don't feel like implementing right now:
    // fn peekable(self) -> Peekable<Self> { self.0.peekable() }
    // fn unzip<A, B, FromA, FromB>(self) -> (FromA, FromB) where FromA: Default + Extend<A>, FromB: Default + Extend<B>, Inner: Iterator<Item = (A, B)> { self.0.unzip() }
    // fn collect<B>(self) -> B where B: FromIterator<Inner::Item>, { self.0.collect() }
    // fn rposition<P>(&mut self, predicate: P) -> Option<usize> where P: FnMut(Self::Item) -> bool, Self: ExactSizeIterator + DoubleEndedIterator { IndexedIter::rposition(self, predicate) }
}

impl<Inner: DoubleEndedIterator, Idx: IndexType> DoubleEndedIterator for IndexedIter<Inner, Idx> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
    // unstable features
    // fn rfold<B, F>(self, accum: B, f: F) -> B where F: FnMut(B, Self::Item) -> B { self.0.rfold(accum, f) }
    // fn rfind<P>(&mut self, predicate: P) -> Option<Self::Item> where P: FnMut(&Self::Item) -> bool { self.0.rfind(predicate) }
}

impl<Inner: ExactSizeIterator, Idx: IndexType> ExactSizeIterator for IndexedIter<Inner, Idx> {
    fn len(&self) -> usize {
        self.0.len()
    }
    // unstable features
    // fn is_empty(&self) -> bool { self.0.is_empty() }
}

impl<Inner: Iterator, Idx: IndexType> Iterator for EnumerateIndex<Inner, Idx> {
    type Item = (Idx, Inner::Item);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|a| {
            let ret = (self.1, a);
            self.1 += 1;
            ret
        })
    }
}

///
/// A Slice with a specific index type (so you don't accidentally use one slice's index on another Vec
/// and can use non-usized indexes).
///
#[derive(Debug)]
#[repr(transparent)]
pub struct IndexedSlice<Elem, Idx: IndexType> {
    marker: PhantomData<Idx>,
    slice: [Elem],
}

impl<T, Idx: IndexType> IndexedSlice<T, Idx> {
    pub fn next_index(&self) -> Idx {
        self.slice.len().into()
    }
    pub fn first_index(&self) -> Idx {
        0.into()
    }
    pub fn last_index(&self) -> Idx {
        (self.slice.len() - 1).into()
    }

    pub const fn len(&self) -> usize {
        self.slice.len()
    }
    pub const fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }
    pub const fn first(&self) -> Option<&T> {
        self.slice.first()
    }
    pub fn first_mut(&mut self) -> Option<&mut T> {
        self.slice.first_mut()
    }
    pub const fn split_first(&self) -> Option<(&T, &[T])> {
        self.slice.split_first()
    }
    pub fn split_first_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        self.slice.split_first_mut()
    }
    pub const fn split_last(&self) -> Option<(&T, &[T])> {
        self.slice.split_last()
    }
    pub fn split_last_mut(&mut self) -> Option<(&mut T, &mut [T])> {
        self.slice.split_last_mut()
    }
    pub const fn last(&self) -> Option<&T> {
        self.slice.last()
    }
    pub fn last_mut(&mut self) -> Option<&mut T> {
        self.slice.last_mut()
    }
    pub const fn first_chunk<const N: usize>(&self) -> Option<&[T; N]> {
        self.slice.first_chunk()
    }
    pub fn first_chunk_mut<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.slice.first_chunk_mut()
    }
    pub const fn split_first_chunk<const N: usize>(&self) -> Option<(&[T; N], &[T])> {
        self.slice.split_first_chunk()
    }
    pub fn split_first_chunk_mut<const N: usize>(&mut self) -> Option<(&mut [T; N], &mut [T])> {
        self.slice.split_first_chunk_mut()
    }
    pub const fn split_last_chunk<const N: usize>(&self) -> Option<(&[T], &[T; N])> {
        self.slice.split_last_chunk()
    }
    pub fn split_last_chunk_mut<const N: usize>(&mut self) -> Option<(&mut [T], &mut [T; N])> {
        self.slice.split_last_chunk_mut()
    }
    pub fn last_chunk<const N: usize>(&self) -> Option<&[T; N]> {
        self.slice.last_chunk()
    }
    pub fn last_chunk_mut<const N: usize>(&mut self) -> Option<&mut [T; N]> {
        self.slice.last_chunk_mut()
    }
    pub fn get(&self, index: Idx) -> Option<&T> {
        self.slice.get(index.into_slice_index())
    }
    pub fn get_mut(&mut self, index: Idx) -> Option<&mut T> {
        self.slice.get_mut(index.into_slice_index())
    }
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I: IntoSliceIndex<[T]>>(&self, index: I) -> &<I::Output as SliceIndex<[T]>>::Output {
        unsafe { self.slice.get_unchecked(index.into_slice_index()) }
    }
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut<I: IntoSliceIndex<[T]>>(&mut self, index: I) -> &mut <I::Output as SliceIndex<[T]>>::Output {
        unsafe { self.slice.get_unchecked_mut(index.into_slice_index()) }
    }
    pub const fn as_ptr(&self) -> *const T {
        self.slice.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.slice.as_mut_ptr()
    }
    pub const fn as_ptr_range(&self) -> Range<*const T> {
        self.slice.as_ptr_range()
    }
    pub fn as_mut_ptr_range(&mut self) -> Range<*mut T> {
        self.slice.as_mut_ptr_range()
    }
    pub fn swap(&mut self, a: Idx, b: Idx) {
        self.slice.swap(a.into(), b.into())
    }
    pub fn reverse(&mut self) {
        self.slice.reverse()
    }
    pub fn iter(&self) -> IndexedIter<Iter<T>, Idx> {
        self.slice.iter().into()
    }
    pub fn iter_mut(&mut self) -> IndexedIter<IterMut<T>, Idx> {
        self.slice.iter_mut().into()
    }
    pub fn windows(&self, size: impl Into<Delta<Idx>>) -> std::slice::Windows<'_, T> {
        self.slice.windows(size.into().into())
    }
    pub fn chunks(&self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::Chunks<'_, T> {
        self.slice.chunks(chunk_size.into().into())
    }
    pub fn chunks_mut(&mut self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::ChunksMut<'_, T> {
        self.slice.chunks_mut(chunk_size.into().into())
    }
    pub fn chunks_exact(&self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::ChunksExact<'_, T> {
        self.slice.chunks_exact(chunk_size.into().into())
    }
    pub fn chunks_exact_mut(&mut self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::ChunksExactMut<'_, T> {
        self.slice.chunks_exact_mut(chunk_size.into().into())
    }
    pub fn rchunks(&self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::RChunks<'_, T> {
        self.slice.rchunks(chunk_size.into().into())
    }
    pub fn rchunks_mut(&mut self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::RChunksMut<'_, T> {
        self.slice.rchunks_mut(chunk_size.into().into())
    }
    pub fn rchunks_exact(&self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::RChunksExact<'_, T> {
        self.slice.rchunks_exact(chunk_size.into().into())
    }
    pub fn rchunks_exact_mut(&mut self, chunk_size: impl Into<Delta<Idx>>) -> std::slice::RChunksExactMut<'_, T> {
        self.slice.rchunks_exact_mut(chunk_size.into().into())
    }
    pub fn chunk_by<F: FnMut(&T, &T) -> bool>(&self, pred: F) -> std::slice::ChunkBy<'_, T, F> {
        self.slice.chunk_by(pred)
    }
    pub fn chunk_by_mut<F: FnMut(&T, &T) -> bool>(&mut self, pred: F) -> std::slice::ChunkByMut<'_, T, F> {
        self.slice.chunk_by_mut(pred)
    }
    // TODO can't use const fn because into()
    pub fn split_at(&self, mid: Idx) -> (&[T], &[T]) {
        self.slice.split_at(mid.into())
    }
    pub fn split_at_mut(&mut self, mid: Idx) -> (&mut [T], &mut [T]) {
        self.slice.split_at_mut(mid.into())
    }
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn split_at_unchecked(&self, mid: Idx) -> (&[T], &[T]) {
        unsafe { self.slice.split_at_unchecked(mid.into()) }
    }
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn split_at_mut_unchecked(&mut self, mid: Idx) -> (&mut [T], &mut [T]) {
        unsafe { self.slice.split_at_mut_unchecked(mid.into()) }
    }
    pub fn split<F: FnMut(&T) -> bool>(&self, pred: F) -> std::slice::Split<'_, T, F> {
        self.slice.split(pred)
    }
    pub fn split_mut<F: FnMut(&T) -> bool>(&mut self, pred: F) -> std::slice::SplitMut<'_, T, F> {
        self.slice.split_mut(pred)
    }
    pub fn split_inclusive<F: FnMut(&T) -> bool>(&self, pred: F) -> std::slice::SplitInclusive<'_, T, F> {
        self.slice.split_inclusive(pred)
    }
    pub fn split_inclusive_mut<F: FnMut(&T) -> bool>(&mut self, pred: F) -> std::slice::SplitInclusiveMut<'_, T, F> {
        self.slice.split_inclusive_mut(pred)
    }
    pub fn rsplit<F: FnMut(&T) -> bool>(&self, pred: F) -> std::slice::RSplit<'_, T, F> {
        self.slice.rsplit(pred)
    }
    pub fn rsplit_mut<F: FnMut(&T) -> bool>(&mut self, pred: F) -> std::slice::RSplitMut<'_, T, F> {
        self.slice.rsplit_mut(pred)
    }
    pub fn splitn<F: FnMut(&T) -> bool>(&self, n: usize, pred: F) -> std::slice::SplitN<'_, T, F> {
        self.slice.splitn(n, pred)
    }
    pub fn splitn_mut<F: FnMut(&T) -> bool>(&mut self, n: usize, pred: F) -> std::slice::SplitNMut<'_, T, F> {
        self.slice.splitn_mut(n, pred)
    }
    pub fn rsplitn<F: FnMut(&T) -> bool>(&self, n: usize, pred: F) -> std::slice::RSplitN<'_, T, F> {
        self.slice.rsplitn(n, pred)
    }
    pub fn rsplitn_mut<F: FnMut(&T) -> bool>(&mut self, n: usize, pred: F) -> std::slice::RSplitNMut<'_, T, F> {
        self.slice.rsplitn_mut(n, pred)
    }
    pub fn contains(&self, x: &T) -> bool where T: PartialEq {
        self.slice.contains(x)
    }
    pub fn starts_with(&self, needle: &[T]) -> bool where T: PartialEq {
        self.slice.starts_with(needle)
    }
    pub fn ends_with(&self, needle: &[T]) -> bool where T: PartialEq {
        self.slice.ends_with(needle)
    }
    // pub fn strip_prefix<P: core::slice::SlicePattern<Item = T>+?Sized>(&self, prefix: &P) -> Option<&[T]> where T: PartialEq {
    //     self.slice.strip_prefix(prefix)
    // }
    // pub fn strip_suffix<P: core::slice::SlicePattern<Item = T>+?Sized>(&self, suffix: &P) -> Option<&[T]> where T: PartialEq {
    //     self.slice.strip_suffix(suffix)
    // }
    pub fn binary_search(&self, x: &T) -> Result<Idx, Idx> where T: Ord {
        self.slice.binary_search(x).map(Into::into).map_err(Into::into)
    }
    pub fn binary_search_by<'a>(&'a self, f: impl FnMut(&'a T) -> Ordering) -> Result<Idx, Idx> {
        self.slice.binary_search_by(f).map(Into::into).map_err(Into::into)
    }
    pub fn binary_search_by_key<'a, B: Ord, F>( &'a self, b: &B, f: impl FnMut(&'a T) -> B) -> Result<Idx, Idx> {
        self.slice.binary_search_by_key(b, f).map(Into::into).map_err(Into::into)
    }
    pub fn sort_unstable(&mut self) where T: Ord {
        self.slice.sort_unstable()
    }
    pub fn sort_unstable_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering) {
        self.slice.sort_unstable_by(compare)
    }
    pub fn sort_unstable_by_key<K: Ord>(&mut self, f: impl FnMut(&T) -> K) {
        self.slice.sort_unstable_by_key(f)
    }
    pub fn select_nth_unstable(&mut self, index: Idx) -> (&mut [T], &mut T, &mut [T]) where T: Ord {
        self.slice.select_nth_unstable(index.into())
    }
    pub fn select_nth_unstable_by(&mut self, index: Idx, compare: impl FnMut(&T, &T) -> Ordering) -> (&mut [T], &mut T, &mut [T]) where T: Ord {
        self.slice.select_nth_unstable_by(index.into(), compare)
    }
    pub fn select_nth_unstable_by_key<K: Ord>(&mut self, index: Idx, f: impl FnMut(&T) -> K) -> (&mut [T], &mut T, &mut [T]) {
        self.slice.select_nth_unstable_by_key(index.into(), f)
    }
    pub fn rotate_left(&mut self, mid: impl Into<Delta<Idx>>) {
        self.slice.rotate_left(mid.into().into())
    }
    pub fn rotate_right(&mut self, mid: impl Into<Delta<Idx>>) {
        self.slice.rotate_right(mid.into().into())
    }
    pub fn fill(&mut self, value: T) where T: Clone {
        self.slice.fill(value)
    }
    pub fn fill_with(&mut self, f: impl FnMut() -> T) {
        self.slice.fill_with(f)
    }
    pub fn clone_from_slice(&mut self, src: &IndexedSlice<T, Idx>) where T: Clone {
        self.slice.clone_from_slice(src.as_raw_slice())
    }
    pub fn copy_from_slice(&mut self, src: &IndexedSlice<T, Idx>) where T: Copy {
        self.slice.copy_from_slice(src.as_raw_slice())
    }
    pub fn copy_within(&mut self, src: impl RangeBounds<Idx>, dest: Idx) where T: Copy {
        self.slice.copy_within(into_bounds(src), dest.into())
    }
    pub fn swap_with_slice(&mut self, other: &mut IndexedSlice<T, Idx>) {
        self.slice.swap_with_slice(other.as_raw_slice_mut())
    }
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn align_to<U>(&self) -> (&[T], &[U], &[T]) {
        unsafe { self.slice.align_to() }
    }
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn align_to_mut<U>(&mut self) -> (&mut [T], &mut [U], &mut [T]) {
        unsafe { self.slice.align_to_mut() }
    }
    pub fn partition_point(&self, pred: impl FnMut(&T) -> bool) -> usize {
        self.slice.partition_point(pred)
    }
    pub fn sort(&mut self) where T: Ord {
        self.slice.sort()
    }
    pub fn sort_by(&mut self, compare: impl FnMut(&T, &T) -> Ordering) {
        self.slice.sort_by(compare)
    }
    pub fn sort_by_key<K: Ord>(&mut self, f: impl FnMut(&T) -> K) {
        self.slice.sort_by_key(f)
    }
    pub fn sort_by_cached_key<K: Ord>(&mut self, f: impl FnMut(&T) -> K) {
        self.slice.sort_by_cached_key(f)
    }
    pub fn to_vec(&self) -> IndexedVec<T, Idx> where T: Clone {
        self.slice.to_vec().into()
    }
    // pub fn into_vec<A: std::alloc::Allocator>(self: Box<[T], A>) -> Vec<T, A> {
    //     self.slice.into_vec()
    // }
    pub fn repeat(&self, n: usize) -> IndexedVec<T, Idx> where T: Copy {
        self.slice.repeat(n).into()
    }
    // pub fn concat<Item: ?Sized>(&self) -> <[T] as std::slice::Concat<Item>>::Output where [T]: Concat<Item> {
    //     self.slice.concat()
    // }
    // pub fn join<Separator>(&self, sep: Separator) -> <[T] as std::slice::Join<Separator>>::Output where [T]: std::slice::Join<Separator> {
    //     self.slice.join(sep)
    // }

    pub fn from_slice(slice: &[T]) -> &Self {
        unsafe { &*(slice as *const [T] as *const Self) }
    }
    pub fn from_mut_slice(slice: &mut [T]) -> &mut Self {
        unsafe { &mut *(slice as *mut [T] as *mut Self) }
    }
    pub fn as_raw_slice(&self) -> &[T] {
        &self.slice
    }
    pub fn as_raw_slice_mut(&mut self) -> &mut [T] {
        &mut self.slice
    }
}

impl<Idx: IndexType> IndexedSlice<u8, Idx> {
    pub const fn is_ascii(&self) -> bool {
        self.slice.is_ascii()
    }
    pub fn eq_ignore_ascii_case(&self, other: &IndexedSlice<u8, Idx>) -> bool {
        self.slice.eq_ignore_ascii_case(other.as_raw_slice())
    }
    pub fn make_ascii_uppercase(&mut self) {
        self.slice.make_ascii_uppercase()
    }
    pub fn make_ascii_lowercase(&mut self) {
        self.slice.make_ascii_lowercase()
    }
    pub fn escape_ascii(&self) -> std::slice::EscapeAscii<'_> {
        self.slice.escape_ascii()
    }
    pub fn utf8_chunks(&self) -> std::str::Utf8Chunks<'_> {
        self.slice.utf8_chunks()
    }
    pub fn to_ascii_uppercase(&self) -> IndexedVec<u8, Idx> {
        self.slice.to_ascii_uppercase().into()
    }
    pub fn to_ascii_lowercase(&self) -> IndexedVec<u8, Idx> {
        self.slice.to_ascii_lowercase().into()
    }
}

fn into_bound(bound: Bound<&impl IndexType>) -> Bound<usize> {
    bound.map(|b| b.into_slice_index())
}
fn into_bounds<Idx: IndexType>(range: impl RangeBounds<Idx>) -> impl RangeBounds<usize> {
    (into_bound(range.start_bound()), into_bound(range.end_bound()))
}

impl<Elem, Idx: IndexType> Index<Idx> for IndexedSlice<Elem, Idx> {
    type Output = Elem;
    fn index(&self, index: Idx) -> &Elem {
        &self.slice[index.into()]
    }
}
impl<Elem, Idx: IndexType> Index<Range<Idx>> for IndexedSlice<Elem, Idx> {
    type Output = [Elem];
    fn index(&self, range: Range<Idx>) -> &[Elem] {
        &self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> Index<RangeFrom<Idx>> for IndexedSlice<Elem, Idx> {
    type Output = [Elem];
    fn index(&self, range: RangeFrom<Idx>) -> &[Elem] {
        &self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> Index<RangeFull> for IndexedSlice<Elem, Idx> {
    type Output = [Elem];
    fn index(&self, range: RangeFull) -> &[Elem] {
        &self.slice[range]
    }
}
impl<Elem, Idx: IndexType> Index<RangeInclusive<Idx>> for IndexedSlice<Elem, Idx> {
    type Output = [Elem];
    fn index(&self, range: RangeInclusive<Idx>) -> &[Elem] {
        &self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> Index<RangeTo<Idx>> for IndexedSlice<Elem, Idx> {
    type Output = [Elem];
    fn index(&self, range: RangeTo<Idx>) -> &[Elem] {
        &self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> Index<RangeToInclusive<Idx>> for IndexedSlice<Elem, Idx> {
    type Output = [Elem];
    fn index(&self, range: RangeToInclusive<Idx>) -> &[Elem] {
        &self.slice[range.into_range()]
    }
}

impl<Elem, Idx: IndexType> IndexMut<Idx> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, index: Idx) -> &mut Elem {
        &mut self.slice[index.into()]
    }
}
impl<Elem, Idx: IndexType> IndexMut<Range<Idx>> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, range: Range<Idx>) -> &mut [Elem] {
        &mut self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> IndexMut<RangeFrom<Idx>> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, range: RangeFrom<Idx>) -> &mut [Elem] {
        &mut self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> IndexMut<RangeFull> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, range: RangeFull) -> &mut [Elem] {
        &mut self.slice[range]
    }
}
impl<Elem, Idx: IndexType> IndexMut<RangeInclusive<Idx>> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, range: RangeInclusive<Idx>) -> &mut [Elem] {
        &mut self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> IndexMut<RangeTo<Idx>> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, range: RangeTo<Idx>) -> &mut [Elem] {
        &mut self.slice[range.into_range()]
    }
}
impl<Elem, Idx: IndexType> IndexMut<RangeToInclusive<Idx>> for IndexedSlice<Elem, Idx> {
    fn index_mut(&mut self, range: RangeToInclusive<Idx>) -> &mut [Elem] {
        &mut self.slice[range.into_range()]
    }
}

impl<Elem: Clone, Idx: IndexType> ToOwned for IndexedSlice<Elem, Idx> {
    type Owned = IndexedVec<Elem, Idx>;
    fn to_owned(&self) -> Self::Owned {
        self.slice.to_vec().into()
    }
}

pub trait IntoSliceIndex<T: ?Sized> {
    type Output: SliceIndex<T>;
    fn into_slice_index(self) -> Self::Output;
}
impl<Idx: IndexType, Elem> IntoSliceIndex<[Elem]> for Idx {
    type Output = usize;
    fn into_slice_index(self) -> Self::Output {
        self.into()
    }
}
impl<Idx: IndexType, Elem> IntoSliceIndex<[Elem]> for Range<Idx> {
    type Output = Range<usize>;
    fn into_slice_index(self) -> Self::Output {
        self.into_range()
    }
}
impl<Idx: IndexType, Elem> IntoSliceIndex<[Elem]> for RangeFrom<Idx> {
    type Output = RangeFrom<usize>;
    fn into_slice_index(self) -> Self::Output {
        self.into_range()
    }
}
impl<Elem> IntoSliceIndex<[Elem]> for RangeFull {
    type Output = RangeFull;
    fn into_slice_index(self) -> Self::Output {
        RangeFull
    }
}
impl<Idx: IndexType, Elem> IntoSliceIndex<[Elem]> for RangeInclusive<Idx> {
    type Output = RangeInclusive<usize>;
    fn into_slice_index(self) -> Self::Output {
        self.into_range()
    }
}


///
/// A Vec with a specific index type (so you don't accidentally use one Vec's index on another Vec).
///
#[derive(Debug, Clone)]
pub struct IndexedVec<Elem, Idx: IndexType>(Vec<Elem>, PhantomData<Idx>);
impl<Elem, Idx: IndexType> IndexedVec<Elem, Idx> {
    pub fn push(&mut self, value: Elem) -> Idx {
        let index = self.next_index();
        self.0.push(value);
        index
    }
    pub fn pop(&mut self) -> Option<Elem> {
        self.0.pop()
    }
    pub fn truncate(&mut self, new_end: Idx) {
        self.0.truncate(new_end.into())
    }
    pub fn insert(&mut self, index: Idx, value: Elem) {
        self.0.insert(index.into(), value)
    }
    pub fn as_raw_vec(&self) -> &Vec<Elem> {
        &self.0
    }
}
impl<Elem, Idx: IndexType> Default for IndexedVec<Elem, Idx> {
    fn default() -> Self {
        Vec::default().into()
    }
}
impl<Elem, Idx: IndexType> Borrow<IndexedSlice<Elem, Idx>> for IndexedVec<Elem, Idx> {
    fn borrow(&self) -> &IndexedSlice<Elem, Idx> {
        self
    }
}
impl<Elem, Idx: IndexType> BorrowMut<IndexedSlice<Elem, Idx>> for IndexedVec<Elem, Idx> {
    fn borrow_mut(&mut self) -> &mut IndexedSlice<Elem, Idx> {
        self
    }
}
impl<Elem, Idx: IndexType> Deref for IndexedVec<Elem, Idx> {
    type Target = IndexedSlice<Elem, Idx>;
    fn deref(&self) -> &Self::Target {
        IndexedSlice::from_slice(self.0.as_slice())
    }
}
impl<Elem, Idx: IndexType> DerefMut for IndexedVec<Elem, Idx> {
    fn deref_mut(&mut self) -> &mut IndexedSlice<Elem, Idx> {
        IndexedSlice::from_mut_slice(self.0.as_mut_slice())
    }
}
impl<Elem, Idx: IndexType> From<Vec<Elem>> for IndexedVec<Elem, Idx> {
    fn from(vec: Vec<Elem>) -> Self {
        IndexedVec(vec, PhantomData)
    }
}
impl<Elem, Idx: IndexType> FromIterator<Elem> for IndexedVec<Elem, Idx> {
    fn from_iter<T: IntoIterator<Item = Elem>>(iter: T) -> Self {
        Vec::from_iter(iter).into()
    }
}
// impl<'a,Elem,Idx:IndexType> IntoIterator for &'a IndexedVec<Elem,Idx> {
//     type Item = &'a Elem;
//     type IntoIter = IndexedIter<Iter<'a, Elem>,Idx>;
//     fn into_iter(mut self) -> Self::IntoIter {
//         self.0.into_iterator()
//     }
// }
// impl<'a,Elem,Idx:IndexType> IntoIterator for &'a mut IndexedVec<Elem,Idx> {
//     type Item = &'a mut Elem;
//     type IntoIter = IndexedIter<IterMut<'a, Elem>,Idx>;
//     fn into_iter(self) -> Self::IntoIter {
//         self.0.into_iterator()
//     }
// }

pub fn to_indexed_cow<Elem: Clone, I: IndexType>(from: Cow<[Elem]>) -> Cow<IndexedSlice<Elem, I>> {
    match from {
        Cow::Borrowed(slice) => Cow::Borrowed(IndexedSlice::from_slice(slice)),
        Cow::Owned(vec) => Cow::Owned(IndexedVec::from(vec)),
    }
}
