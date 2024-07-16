use std::simd::{Mask, MaskElement};
use derive_more::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, From, Into, Not};

#[derive(Default, Debug, Clone, Copy, From, Into, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, PartialEq, Eq)]
#[repr(transparent)]
pub struct ByteMask(pub u64);

impl PartialEq<u64> for ByteMask {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl ByteMask {
    #[inline]
    pub fn any(&self) -> bool {
        self.0 != 0
    }
    #[inline]
    pub fn all(&self) -> bool {
        self.0 == u64::MAX
    }
}

impl<T: MaskElement> From<Mask<T, 64>> for ByteMask {
    #[inline]
    fn from(mask: Mask<T, 64>) -> Self {
        ByteMask(mask.to_bitmask())
    }
}

