use std::ops::Shr;

use super::{Mask64, BLOCK_SIZE};

pub struct Strings {
    /// real quotes (non-escaped ones)
    pub quote: Mask64,
    /// string characters (includes start quote but not end quote)
    pub start_quote_and_string: Mask64,
}

///
/// Takes a mask of quotes, and returns a mask of characters inside each pair of quotes.
/// 
/// i.e. between the first and second, third and fourth, fifth and sixth, etc.
/// 
#[repr(transparent)]
pub struct StringScanner {
    pub still_in_string: Mask64,
}

impl StringScanner {
    #[inline]
    pub fn next(&mut self, quote: Mask64) -> Strings {
        //
        // prefix_xor flips on bits inside the string (and flips off the end quote).
        //
        // Then we xor with still_in_string: if we were in a string already, its effect is flipped
        // (characters inside strings are outside, and characters outside strings are inside).
        //
        let start_quote_and_string = Self::prefix_xor(quote) ^ self.still_in_string;

        //
        // Check if we're still inside a string at the end of the box so the next block will know
        //
        // Shift right arithmetically: if the high bit is 1, then it will be all 1's,
        // otherwise it will be all 0's.
        self.still_in_string = (start_quote_and_string as i64).shr(BLOCK_SIZE-1) as u64;

        Strings { quote, start_quote_and_string }
    }

    #[inline]
    fn prefix_xor(delimiters: Mask64) -> Mask64 {
        // TODO AI did this and I don't trust it
        let mut prefix_xor = delimiters;
        prefix_xor ^= prefix_xor >> 1;
        prefix_xor ^= prefix_xor >> 2;
        prefix_xor ^= prefix_xor >> 4;
        prefix_xor ^= prefix_xor >> 8;
        prefix_xor ^= prefix_xor >> 16;
        prefix_xor ^= prefix_xor >> 32;
        prefix_xor
    }
}

impl Strings {
    /// Real (non-backslashed) quotes
    #[inline]
    pub fn quote(&self) -> Mask64 { self.quote }
    /// Only characters inside the string (not including the quotes)
    #[inline]
    pub fn string_content(&self) -> Mask64 { self.start_quote_and_string & !self.quote }
    /// Tail of string (everything except the start quote)
    #[inline]
    pub fn end_quote_and_string(&self) -> Mask64 { self.start_quote_and_string ^ self.quote }
    /// Return a mask of whether the given characters are inside a string (only works on non-quotes)
    #[inline]
    pub fn non_quote_inside_string(&self, mask: Mask64) -> Mask64 { mask & self.start_quote_and_string }
    /// Return a mask of whether the given characters are inside a string (only works on non-quotes)
    #[inline]
    pub fn non_quote_outside_string(&self, mask: Mask64) -> Mask64 { mask & !self.start_quote_and_string }
}

