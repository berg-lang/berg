use super::{Mask64, BLOCK_SIZE};

#[repr(transparent)]
pub struct SequenceScanner {
    pub member: Mask64,
}

#[repr(transparent)]
pub struct Sequence {
    /// Mask showing which members are *preceded* by a member. This is used by `start()`, `end()`,
    /// and `start_and_end()` to retrieve the actual start and end.
    pub preceded_by_member: Mask64,
}

impl SequenceScanner {
    #[inline]
    pub fn next(&mut self, member: Mask64) -> Sequence {
        let preceded_by_member = (member >> 1) | (self.member << (BLOCK_SIZE-1));
        self.member = member; // Remember the member for next time
        Sequence { preceded_by_member }
    }
}

impl Sequence {
    // Get the members of the sequence, which are stored in the scanner after calling next()
    // (since they are used in the next iteration).
    #[inline]
    pub fn member(&self, scanner: &SequenceScanner) -> Mask64 {
        scanner.member
    }

    // Get the start of each sequence.
    #[inline]
    pub fn start(&self, scanner: &SequenceScanner) -> Mask64 {
        self.member(scanner) & !self.preceded_by_member
    }

    /// Get the end of each sequence (the one right *after* the sequence).
    #[inline]
    pub fn end(&self, scanner: &SequenceScanner) -> Mask64 {
        self.preceded_by_member & !self.member(scanner)
    }

    /// Get a mask including the start and end of each sequence.
    #[inline]
    pub fn start_and_end(&self, scanner: &SequenceScanner) -> Mask64 {
        self.member(scanner) ^ self.preceded_by_member
    }

    /// Get the tail part of each sequence (all parts of all sequences except the heads).
    #[inline]
    pub fn tail(&self, scanner: &SequenceScanner) -> Mask64 {
        self.member(scanner) & self.preceded_by_member
    }
}