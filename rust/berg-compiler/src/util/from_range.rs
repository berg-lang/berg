use std::ops::Add;
use std::ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

pub trait IntoRange<ToIndex> {
    type Output: RangeBounds<ToIndex>;
    fn into_range(self) -> Self::Output;
}

pub trait FromRange<FromIndex> {
    type From: RangeBounds<FromIndex>;
    fn from_range(from: Self::From) -> Self;
}

impl<FromIndex, ToIndex: From<FromIndex>> FromRange<FromIndex> for Range<ToIndex>            { type From = Range<FromIndex>;            fn from_range(from: Self::From) -> Self { Self { start: ToIndex::from(from.start), end: ToIndex::from(from.end) } } }
impl<FromIndex, ToIndex: From<FromIndex>> FromRange<FromIndex> for RangeFrom<ToIndex>        { type From = RangeFrom<FromIndex>;        fn from_range(from: Self::From) -> Self { Self { start: ToIndex::from(from.start) } } }
impl<FromIndex: Copy, ToIndex: From<FromIndex>> FromRange<FromIndex> for RangeInclusive<ToIndex>   { type From = RangeInclusive<FromIndex>;   fn from_range(from: Self::From) -> Self { Self::new(ToIndex::from(*from.end()), ToIndex::from(*from.end())) } }
impl<FromIndex, ToIndex: From<FromIndex>> FromRange<FromIndex> for RangeTo<ToIndex>          { type From = RangeTo<FromIndex>;          fn from_range(from: Self::From) -> Self { Self { end: ToIndex::from(from.end) } } }
impl<FromIndex, ToIndex: From<FromIndex>> FromRange<FromIndex> for RangeToInclusive<ToIndex> { type From = RangeToInclusive<FromIndex>; fn from_range(from: Self::From) -> Self { Self { end: ToIndex::from(from.end) } } }
impl<FromIndex> FromRange<FromIndex> for RangeFull { type From = RangeFull; fn from_range(from: RangeFull) -> RangeFull { from } }

impl<FromIndex, ToIndex: From<FromIndex>> IntoRange<ToIndex> for Range<FromIndex>            { type Output = Range<ToIndex>;            fn into_range(self) -> Self::Output { Self::Output { start: ToIndex::from(self.start), end: ToIndex::from(self.end) } } }
impl<FromIndex, ToIndex: From<FromIndex>> IntoRange<ToIndex> for RangeFrom<FromIndex>        { type Output = RangeFrom<ToIndex>;        fn into_range(self) -> Self::Output { Self::Output { start: ToIndex::from(self.start) } } }
impl<FromIndex: Copy, ToIndex: From<FromIndex>> IntoRange<ToIndex> for RangeInclusive<FromIndex>   { type Output = RangeInclusive<ToIndex>;   fn into_range(self) -> Self::Output { Self::Output::new(ToIndex::from(*self.start()), ToIndex::from(*self.end())) } }
impl<FromIndex, ToIndex: From<FromIndex>> IntoRange<ToIndex> for RangeTo<FromIndex>          { type Output = RangeTo<ToIndex>;          fn into_range(self) -> Self::Output { Self::Output { end: ToIndex::from(self.end) } } }
impl<FromIndex, ToIndex: From<FromIndex>> IntoRange<ToIndex> for RangeToInclusive<FromIndex> { type Output = RangeToInclusive<ToIndex>; fn into_range(self) -> Self::Output { Self::Output { end: ToIndex::from(self.end) } } }
impl<FromIndex> IntoRange<FromIndex> for RangeFull { type Output = RangeFull; fn into_range(self) -> RangeFull { self } }

pub trait ExplicitRange<I>: RangeBounds<I> {
    fn explicit_range(&self, len: I) -> Range<I> { Range { start: self.explicit_start_bound(), end: self.explicit_end_bound(len) } }
    fn explicit_start_bound(&self) -> I;
    fn explicit_end_bound(&self, len: I) -> I;
}

impl<I: Copy> ExplicitRange<I> for Range<I> {
    fn explicit_start_bound(&self) -> I { self.start }
    fn explicit_end_bound(&self, _: I) -> I { self.end }
}
impl<I: Copy+Add<usize, Output=I>> ExplicitRange<I> for RangeInclusive<I> {
    fn explicit_start_bound(&self) -> I { *self.start() }
    fn explicit_end_bound(&self, _: I) -> I { *self.end() + 1 }
}
impl<I: Copy> ExplicitRange<I> for RangeFrom<I> {
    fn explicit_start_bound(&self) -> I { self.start }
    fn explicit_end_bound(&self, len: I) -> I { len }
}
impl<I: Copy+From<usize>> ExplicitRange<I> for RangeTo<I> {
    fn explicit_start_bound(&self) -> I { I::from(0) }
    fn explicit_end_bound(&self, _: I) -> I { self.end }
}
impl<I: Copy+From<usize>+Add<usize, Output=I>> ExplicitRange<I> for RangeToInclusive<I> {
    fn explicit_start_bound(&self) -> I { I::from(0) }
    fn explicit_end_bound(&self, _: I) -> I { self.end + 1 }
}
impl<I: Copy+From<usize>> ExplicitRange<I> for RangeFull {
    fn explicit_start_bound(&self) -> I { I::from(0) }
    fn explicit_end_bound(&self, len: I) -> I { len }
}
