#[derive(
    Copy, Clone, Debug, PartialEq, Eq,
    derive_more::AsRef, derive_more::AsMut, derive_more::From, derive_more::Into,
    derive_more::BitAnd, derive_more::BitOr, derive_more::BitXor, derive_more::Not,
    derive_more::BitAndAssign, derive_more::BitOrAssign, derive_more::BitXorAssign,
)]
struct Mask64(pub u64);

impl Mask64 {
    #[inline]
    pub fn new(value: u64) -> Self {
        Mask64(value)
    }

    #[inline]
    pub fn next(&mut self, new_mask: Mask64) {
        self.0 = new_mask.0
    }
}