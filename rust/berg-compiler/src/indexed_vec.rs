use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;
use std::ops::RangeTo;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;
use std::ops::SubAssign;
use std::marker::PhantomData;

// index_type and indexed_vec work together to let you use a custom type
// (like TokenIndex) to index the vector, and disallow any other type (like usize
// or SourceIndex) from accessing it, so that you can be sure you are accessing
// the thing you think you are!
//
// Think of index_type (TokenIndex/SourceIndex/ByteIndex) as your number and
// the IndexedVec as Vec<Token>/Vec<Source>, and you will be good.

#[macro_export]
macro_rules! index_type {
    ($(pub struct $name:ident(pub $($type:tt)*) <= $max:expr;)*) => {
        use indexed_vec::IndexType;
        use std::fmt;
        use std::ops::Add;
        use std::ops::AddAssign;
        use std::ops::Sub;
        use std::ops::SubAssign;
        use std::cmp::Ordering;
        $(
            #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Ord, PartialOrd)]
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
            impl IndexType for $name {}
            impl $name { pub const MAX: $name = $name($max); }
            impl From<usize> for $name { fn from(size: usize) -> Self { $name(size as $($type)*) } }
            impl From<$name> for usize { fn from(size: $name) -> Self { size.0 as usize } }
            impl Add<usize> for $name { type Output = Self; fn add(self, value: usize) -> Self { $name(self.0 + Self::from(value).0) } }
            impl Sub<usize> for $name { type Output = Self; fn sub(self, value: usize) -> Self { $name(self.0 - Self::from(value).0) } }
            impl Sub<$name> for $name { type Output = Self; fn sub(self, value: $name) -> Self { $name(self.0 - value.0) } }
            impl AddAssign<usize> for $name { fn add_assign(&mut self, value: usize) { *self = *self + value } }
            impl SubAssign<usize> for $name { fn sub_assign(&mut self, value: usize) { *self = *self - value } }
        )*
    }
}
pub trait IndexType: Into<usize>+From<usize>+PartialOrd+PartialEq+Copy+AddAssign<usize>+SubAssign<usize>+Add<usize,Output=Self>+Sub<usize,Output=Self> {}

///
/// A Vec with a specific index type (so you don't accidentally use one Vec's index on another Vec).
///
#[derive(Debug, Clone)]
pub struct IndexedVec<Elem, Ind: IndexType>(Vec<Elem>, PhantomData<Ind>);
impl<Elem, Ind: IndexType> IndexedVec<Elem, Ind> {
    pub fn with_capacity(size: usize) -> Self {
        IndexedVec(Vec::with_capacity(size), PhantomData)
    }
    pub fn len(&self) -> Ind {
        Ind::from(self.0.len())
    }
    pub fn push(&mut self, elem: Elem) {
        self.0.push(elem)
    }
}

impl<Elem, Ind: IndexType> Default for IndexedVec<Elem, Ind> {
    fn default() -> Self {
        IndexedVec(vec![], PhantomData)
    }
}

impl<Elem, Ind: IndexType> Index<Ind> for IndexedVec<Elem, Ind> {
    type Output = Elem;
    fn index(&self, index: Ind) -> &Elem {
        let index: usize = index.into();
        &self.0[index]
    }
}
impl<Elem, Ind: IndexType> Index<Range<Ind>> for IndexedVec<Elem, Ind> {
    type Output = [Elem];
    fn index(&self, range: Range<Ind>) -> &[Elem] {
        let start: usize = range.start.into();
        let end: usize = range.end.into();
        &self.0[Range { start, end }]
    }
}
impl<Elem, Ind: IndexType> Index<RangeTo<Ind>> for IndexedVec<Elem, Ind> {
    type Output = [Elem];
    fn index(&self, range: RangeTo<Ind>) -> &[Elem] {
        let end: usize = range.end.into();
        &self.0[RangeTo { end }]
    }
}
impl<Elem, Ind: IndexType> Index<RangeFrom<Ind>> for IndexedVec<Elem, Ind> {
    type Output = [Elem];
    fn index(&self, range: RangeFrom<Ind>) -> &[Elem] {
        let start: usize = range.start.into();
        &self.0[RangeFrom { start }]
    }
}
impl<Elem, Ind: IndexType> Index<RangeFull> for IndexedVec<Elem, Ind> {
    type Output = [Elem];
    fn index(&self, range: RangeFull) -> &[Elem] {
        &self.0[range]
    }
}

impl<Elem, Ind: IndexType> IndexMut<Ind> for IndexedVec<Elem, Ind> {
    fn index_mut(&mut self, index: Ind) -> &mut Elem {
        let index: usize = index.into();
        &mut self.0[index]
    }
}
