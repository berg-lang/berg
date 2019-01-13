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

pub trait BoundedRange<I>: RangeBounds<I> {
    fn bounded_range(&self, len: I) -> Range<I> { Range { start: self.bounded_start_bound(), end: self.bounded_end_bound(len) } }
    fn bounded_start_bound(&self) -> I;
    fn bounded_end_bound(&self, len: I) -> I;
}

impl<I: Copy> BoundedRange<I> for Range<I> {
    fn bounded_start_bound(&self) -> I { self.start }
    fn bounded_end_bound(&self, _: I) -> I { self.end }
}
impl<I: Copy+Add<usize, Output=I>> BoundedRange<I> for RangeInclusive<I> {
    fn bounded_start_bound(&self) -> I { *self.start() }
    fn bounded_end_bound(&self, _: I) -> I { *self.end() + 1 }
}
impl<I: Copy> BoundedRange<I> for RangeFrom<I> {
    fn bounded_start_bound(&self) -> I { self.start }
    fn bounded_end_bound(&self, len: I) -> I { len }
}
impl<I: Copy+From<usize>> BoundedRange<I> for RangeTo<I> {
    fn bounded_start_bound(&self) -> I { I::from(0) }
    fn bounded_end_bound(&self, _: I) -> I { self.end }
}
impl<I: Copy+From<usize>+Add<usize, Output=I>> BoundedRange<I> for RangeToInclusive<I> {
    fn bounded_start_bound(&self) -> I { I::from(0) }
    fn bounded_end_bound(&self, _: I) -> I { self.end + 1 }
}
impl<I: Copy+From<usize>> BoundedRange<I> for RangeFull {
    fn bounded_start_bound(&self) -> I { I::from(0) }
    fn bounded_end_bound(&self, len: I) -> I { len }
}
