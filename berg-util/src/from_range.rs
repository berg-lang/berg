use std::ops::{
    Add, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

pub trait IntoRange<ToIndex> {
    type Output: RangeBounds<ToIndex>;
    fn into_range(self) -> Self::Output;
}

#[allow(dead_code)]
pub trait FromRange<FromIndex> {
    type From: RangeBounds<FromIndex>;
    fn from_range(from: Self::From) -> Self;
}

impl<ToIndex, FromIndex: Into<ToIndex>> FromRange<FromIndex> for Range<ToIndex> {
    type From = Range<FromIndex>;
    fn from_range(from: Self::From) -> Self {
        Self {
            start: from.start.into(),
            end: from.end.into(),
        }
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> FromRange<FromIndex> for RangeFrom<ToIndex> {
    type From = RangeFrom<FromIndex>;
    fn from_range(from: Self::From) -> Self {
        Self {
            start: from.start.into(),
        }
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> FromRange<FromIndex> for RangeInclusive<ToIndex> {
    type From = RangeInclusive<FromIndex>;
    fn from_range(from: Self::From) -> Self {
        let (start, end) = from.into_inner();
        Self::new(start.into(), end.into())
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> FromRange<FromIndex> for RangeTo<ToIndex> {
    type From = RangeTo<FromIndex>;
    fn from_range(from: Self::From) -> Self {
        Self {
            end: from.end.into(),
        }
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> FromRange<FromIndex> for RangeToInclusive<ToIndex> {
    type From = RangeToInclusive<FromIndex>;
    fn from_range(from: Self::From) -> Self {
        Self {
            end: from.end.into(),
        }
    }
}
impl<FromIndex> FromRange<FromIndex> for RangeFull {
    type From = RangeFull;
    fn from_range(from: RangeFull) -> RangeFull {
        from
    }
}

impl<ToIndex, FromIndex: Into<ToIndex>> IntoRange<ToIndex> for Range<FromIndex> {
    type Output = Range<ToIndex>;
    fn into_range(self) -> Self::Output {
        Self::Output {
            start: self.start.into(),
            end: self.end.into(),
        }
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> IntoRange<ToIndex> for RangeFrom<FromIndex> {
    type Output = RangeFrom<ToIndex>;
    fn into_range(self) -> Self::Output {
        Self::Output {
            start: self.start.into(),
        }
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> IntoRange<ToIndex> for RangeInclusive<FromIndex> {
    type Output = RangeInclusive<ToIndex>;
    fn into_range(self) -> Self::Output {
        let (start, end) = self.into_inner();
        Self::Output::new(start.into(), end.into())
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> IntoRange<ToIndex> for RangeTo<FromIndex> {
    type Output = RangeTo<ToIndex>;
    fn into_range(self) -> Self::Output {
        Self::Output {
            end: self.end.into(),
        }
    }
}
impl<ToIndex, FromIndex: Into<ToIndex>> IntoRange<ToIndex> for RangeToInclusive<FromIndex> {
    type Output = RangeToInclusive<ToIndex>;
    fn into_range(self) -> Self::Output {
        Self::Output {
            end: self.end.into(),
        }
    }
}
impl<FromIndex> IntoRange<FromIndex> for RangeFull {
    type Output = RangeFull;
    fn into_range(self) -> RangeFull {
        self
    }
}

pub trait BoundedRange<I>: RangeBounds<I> {
    fn bounded_range(&self, len: I) -> Range<I> {
        Range {
            start: self.bounded_start_bound(),
            end: self.bounded_end_bound(len),
        }
    }
    fn bounded_start_bound(&self) -> I;
    fn bounded_end_bound(&self, len: I) -> I;
}

impl<I: Copy> BoundedRange<I> for Range<I> {
    fn bounded_start_bound(&self) -> I {
        self.start
    }
    fn bounded_end_bound(&self, _: I) -> I {
        self.end
    }
}
impl<I: Copy + Add<usize, Output = I>> BoundedRange<I> for RangeInclusive<I> {
    fn bounded_start_bound(&self) -> I {
        *self.start()
    }
    fn bounded_end_bound(&self, _: I) -> I {
        *self.end() + 1
    }
}
impl<I: Copy> BoundedRange<I> for RangeFrom<I> {
    fn bounded_start_bound(&self) -> I {
        self.start
    }
    fn bounded_end_bound(&self, len: I) -> I {
        len
    }
}
impl<I: Copy + From<usize>> BoundedRange<I> for RangeTo<I> {
    fn bounded_start_bound(&self) -> I {
        I::from(0)
    }
    fn bounded_end_bound(&self, _: I) -> I {
        self.end
    }
}
impl<I: Copy + From<usize> + Add<usize, Output = I>> BoundedRange<I> for RangeToInclusive<I> {
    fn bounded_start_bound(&self) -> I {
        I::from(0)
    }
    fn bounded_end_bound(&self, _: I) -> I {
        self.end + 1
    }
}
impl<I: Copy + From<usize>> BoundedRange<I> for RangeFull {
    fn bounded_start_bound(&self) -> I {
        I::from(0)
    }
    fn bounded_end_bound(&self, len: I) -> I {
        len
    }
}
