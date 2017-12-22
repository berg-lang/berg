use std::borrow::Cow;
use std::ops::RangeFrom;
use std::slice::IterMut;
use std::slice::Iter;
use std::cmp::Ordering;
use std::borrow::{Borrow,BorrowMut};
use std::fmt;
use std::ops::{Add,AddAssign,Deref,DerefMut,Index,IndexMut,Range,Sub,SubAssign};
use std::marker::PhantomData;
use std::mem;

// index_type and util::indexed_vec work together to let you use a custom type
// (like TokenIndex) to index the vector, and disallow any other type (like usize
// or SourceIndex) from accessing it, so that you can be sure you are accessing
// the thing you think you are!
//
// Think of index_type (TokenIndex/SourceIndex/ByteIndex) as your number and
// the IndexedVec as Vec<Token>/Vec<Source>, and you will be good.

#[macro_export]
macro_rules! index_type {
    ($(pub struct $name:ident(pub $($type:tt)*) <= $max:expr;)*) => {
        use util::indexed_vec::{Delta,IndexType};
        use std::fmt;
        use std::ops::{Add,AddAssign,Sub,SubAssign};
        use std::cmp::Ordering;
        $(
            #[derive(Debug,Copy,Clone,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]
            pub struct $name(pub $($type)*);
            impl PartialEq<usize> for $name {
                fn eq(&self, other: &usize) -> bool { (self.0 as usize).eq(other) }
            }
            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
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
    }
}

#[derive(Debug,Copy,Clone,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Delta<IndexType>(pub IndexType);
impl<T: IndexType> fmt::Display for Delta<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: IndexType> From<Delta<T>> for usize { fn from(size: Delta<T>) -> Self { size.0.into() } }
impl<T: IndexType> From<usize> for Delta<T> { fn from(size: usize) -> Self { Delta(T::from(size)) } }
impl<T: IndexType> PartialEq<usize> for Delta<T> {
    fn eq(&self, other: &usize) -> bool { self.0.eq(other) }
}
impl<T: IndexType> PartialOrd<usize> for Delta<T> {
    fn partial_cmp(&self, other: &usize) -> Option<Ordering> { self.0.partial_cmp(other) }
    fn lt(&self, other: &usize) -> bool { self.0 < *other }
    fn le(&self, other: &usize) -> bool { self.0 <= *other }
    fn gt(&self, other: &usize) -> bool { self.0 > *other }
    fn ge(&self, other: &usize) -> bool { self.0 >= *other }
}

pub trait IndexType: Copy+Clone+fmt::Display+
    Into<usize>+From<usize>+
    PartialOrd+PartialEq+
    PartialOrd<usize>+PartialEq<usize>+
    AddAssign<usize>+SubAssign<usize>+Add<usize,Output=Self>+Sub<usize,Output=Self>
{
}

///
/// A Slice with a specific index type (so you don't accidentally use one slice's index on another Vec
/// and can use non-usized indexes).
///
pub struct IndexedSlice<Elem, Index: IndexType> {
    marker: PhantomData<Index>,
    slice: [Elem],
}
impl<Elem, Index: IndexType> IndexedSlice<Elem, Index> {
    pub fn len(&self) -> Index { self.slice.len().into() }
    pub fn get(&self, index: Index) -> Option<&Elem> { self.slice.get(index.into()) }
    pub fn iter(&self) -> Iter<Elem> { self.slice.iter() }
    pub fn iter_mut(&mut self) -> IterMut<Elem> { self.slice.iter_mut() }
    pub fn is_empty(&self) -> bool { self.slice.is_empty() }
    pub fn from_slice(slice: &[Elem]) -> &Self { unsafe { mem::transmute(slice) } }
    pub fn from_mut_slice(slice: &mut [Elem]) -> &mut Self { unsafe { mem::transmute(slice) } }
    pub fn first(&self) -> Option<&Elem> { self.slice.first() }
    pub fn last(&self) -> Option<&Elem> { self.slice.last() }
    pub fn last_mut(&mut self) -> Option<&mut Elem> { self.slice.last_mut() }
    pub fn as_raw_slice(&self) -> &[Elem] { &self.slice }
}

impl<Elem, I: IndexType> Index<I> for IndexedSlice<Elem,I> {
    type Output = Elem;
    fn index(&self, index: I) -> &Elem {
        &self.slice[index.into()]
    }
}
impl<Elem, I: IndexType> IndexMut<I> for IndexedSlice<Elem,I> {
    fn index_mut(&mut self, index: I) -> &mut Elem {
        &mut self.slice[index.into()]
    }
}
impl<Elem, I: IndexType> Index<Range<I>> for IndexedSlice<Elem,I> {
    type Output = [Elem];
    fn index(&self, range: Range<I>) -> &[Elem] {
        &self.slice[range.start.into()..range.end.into()]
    }
}
impl<Elem, I: IndexType> Index<RangeFrom<I>> for IndexedSlice<Elem,I> {
    type Output = [Elem];
    fn index(&self, range: RangeFrom<I>) -> &[Elem] {
        &self.slice[range.start.into()..]
    }
}
impl<Elem, I: IndexType> IndexMut<Range<I>> for IndexedSlice<Elem,I> {
    fn index_mut(&mut self, range: Range<I>) -> &mut [Elem] {
        &mut self.slice[range.start.into()..range.end.into()]
    }
}
impl<Elem, I: IndexType> IndexMut<RangeFrom<I>> for IndexedSlice<Elem,I> {
    fn index_mut(&mut self, range: RangeFrom<I>) -> &mut [Elem] {
        &mut self.slice[range.start.into()..]
    }
}
impl<'a, Elem, I: IndexType> Index<&'a Range<I>> for IndexedSlice<Elem,I> {
    type Output = [Elem];
    fn index<'s>(&'s self, range: &'a Range<I>) -> &'s [Elem] {
        &self.slice[range.start.into()..range.end.into()]
    }
}
impl<'a, Elem, I: IndexType> Index<&'a RangeFrom<I>> for IndexedSlice<Elem,I> {
    type Output = [Elem];
    fn index<'s>(&'s self, range: &'a RangeFrom<I>) -> &'s [Elem] {
        &self.slice[range.start.into()..]
    }
}
impl<'a, Elem, I: IndexType> IndexMut<&'a Range<I>> for IndexedSlice<Elem,I> {
    fn index_mut(&mut self, range: &'a Range<I>) -> &mut [Elem] {
        &mut self.slice[range.start.into()..range.end.into()]
    }
}
impl<'a, Elem, I: IndexType> IndexMut<&'a RangeFrom<I>> for IndexedSlice<Elem,I> {
    fn index_mut(&mut self, range: &'a RangeFrom<I>) -> &mut [Elem] {
        &mut self.slice[range.start.into()..]
    }
}

impl<Elem: Clone, I: IndexType> ToOwned for IndexedSlice<Elem,I> {
    type Owned = IndexedVec<Elem,I>;
    fn to_owned(&self) -> Self::Owned { (&self.slice).to_vec().into() }
}

///
/// A Vec with a specific index type (so you don't accidentally use one Vec's index on another Vec).
///
#[derive(Debug, Clone)]
pub struct IndexedVec<Elem, I: IndexType> {
    inner: Vec<Elem>,
    marker: PhantomData<I>,
}
impl<Elem, I: IndexType> IndexedVec<Elem,I> {
    pub fn push(&mut self, value: Elem) -> I { self.inner.push(value); self.len()-1 }
    pub fn pop(&mut self) -> Option<Elem> { self.inner.pop() }
    pub fn truncate(&mut self, new_end: I) { self.inner.truncate(new_end.into()) }
    pub fn insert(&mut self, index: I, value: Elem) { self.inner.insert(index.into(), value) }
    pub fn as_raw_vec(&self) -> &Vec<Elem> { &self.inner }
}
impl<Elem, I: IndexType> Default for IndexedVec<Elem,I> {
    fn default() -> Self { Vec::default().into() }
}
impl<Elem, I: IndexType> Borrow<IndexedSlice<Elem,I>> for IndexedVec<Elem,I> {
    fn borrow(&self) -> &IndexedSlice<Elem,I> { self }
}
impl<Elem, I: IndexType> BorrowMut<IndexedSlice<Elem,I>> for IndexedVec<Elem,I> {
    fn borrow_mut(&mut self) -> &mut IndexedSlice<Elem,I> { self }
}
impl<Elem, I: IndexType> Deref for IndexedVec<Elem,I> {
    type Target = IndexedSlice<Elem, I>;
    fn deref(&self) -> &Self::Target { IndexedSlice::from_slice(self.inner.as_slice()) }
}
impl<Elem, I: IndexType> DerefMut for IndexedVec<Elem,I> {
    fn deref_mut(&mut self) -> &mut IndexedSlice<Elem, I> { IndexedSlice::from_mut_slice(self.inner.as_mut_slice()) }
}
impl<Elem, I: IndexType> From<Vec<Elem>> for IndexedVec<Elem,I> {
    fn from(vec: Vec<Elem>) -> Self { IndexedVec { inner: vec, marker: PhantomData } }
}

pub fn to_indexed_cow<Elem: Clone, I: IndexType>(from: Cow<[Elem]>) -> Cow<IndexedSlice<Elem,I>> {
    match from {
        Cow::Borrowed(slice) => Cow::Borrowed(IndexedSlice::from_slice(slice)),
        Cow::Owned(vec) => Cow::Owned(IndexedVec::from(vec))
    }
}
