use super::{Mask64, BLOCK_SIZE};

pub struct SequenceScanner {
    pub prev_member: Mask64,
}

pub struct Sequence {
    /// Mask of characters in the sequence. This is the original input that was passed in.
    pub member: Mask64,
    /// Mask showing which members are *preceded* by a member. This is used by `start()`, `end()`,
    /// and `start_and_end()` to retrieve the actual start and end.
    pub preceded_by_member: Mask64,
}

impl SequenceScanner {
    #[inline]
    pub fn next(&mut self, member: Mask64) -> Sequence {
        let preceded_by_member = member >> 1 | self.prev_member << (BLOCK_SIZE-1);
        self.prev_member = member;
        Sequence { member, preceded_by_member }
    }
}

impl Sequence {
    // Get the start of each sequence.
    #[inline]
    pub fn start(&self) -> Mask64 {
        self.member & !self.preceded_by_member
    }

    /// Get the end of each sequence (the one right *after* the sequence).
    #[inline]
    pub fn end(&self) -> Mask64 {
        self.preceded_by_member & !self.member
    }

    /// Get a mask including the start and end of each sequence.
    #[inline]
    pub fn start_and_end(&self) -> Mask64 {
        self.member ^ self.preceded_by_member
    }

    /// Get the tail part of each sequence (all parts of all sequences except the heads).
    #[inline]
    pub fn tail(&self) -> Mask64 {
        self.member & self.preceded_by_member
    }
}