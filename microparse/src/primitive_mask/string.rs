use std::ops::Shr;

use crate::arch::native::prefix_xor;

use super::{Mask64, BLOCK_SIZE, LAST};

#[repr(transparent)]
pub struct Strings {
    /// real quotes (non-escaped ones)
    pub quote: Mask64,
}

///
/// Takes a mask of quotes, and returns a mask of characters inside each pair of quotes.
/// 
/// i.e. between the first and second, third and fourth, fifth and sixth, etc.
/// 
#[repr(transparent)]
pub struct StringScanner {
    pub start_quote_and_string: Mask64,
}

impl Default for StringScanner {
    #[inline(always)]
    fn default() -> Self {
        Self { start_quote_and_string: 0 }
    }
}

impl StringScanner {
    #[inline(always)]
    pub fn next(&mut self, quote: Mask64) -> Strings {
        //
        // prefix_xor flips on bits inside the string (and flips off the end quote).
        //
        // Then we xor with still_in_string: if we were in a string already, its effect is flipped
        // (characters inside strings are outside, and characters outside strings are inside).
        //
        let local_in_string = prefix_xor(quote);
        // Shift right arithmetically: if the high bit of the last start_quote_and_string is 1, then
        // still_in_string will be all 1's, otherwise it will be all 0's.
        let still_in_string = (self.start_quote_and_string as i64).shr(BLOCK_SIZE-1) as u64;
        self.start_quote_and_string = local_in_string ^ still_in_string;

        Strings { quote }
    }

    /// Tell whether we are still in a string.
    #[inline(always)]
    pub fn still_in_string(&self) -> bool {
        // Test whether the last bit of the previous start_quote_and_string is 1.
        (self.start_quote_and_string & LAST) != 0
    }
}

impl Strings {
    // Start quotes.
    #[inline(always)]
    pub fn start_quote(&self, scanner: &StringScanner) -> Mask64 { scanner.start_quote_and_string & self.quote }
    /// End quotes.
    #[inline(always)]
    pub fn end_quote(&self, scanner: &StringScanner) -> Mask64 { scanner.start_quote_and_string & !self.quote }
    /// Only characters inside the string (not including the quotes)
    #[inline(always)]
    pub fn string_content(&self, scanner: &StringScanner) -> Mask64 { scanner.start_quote_and_string & !self.quote }
    /// Tail of string (everything except the start quote)
    #[inline(always)]
    pub fn end_quote_and_string(&self, scanner: &StringScanner) -> Mask64 { scanner.start_quote_and_string ^ self.quote }
    /// Return a mask of whether the given characters are inside a string (only works on non-quotes)
    #[inline(always)]
    pub fn non_quote_inside_string(&self, scanner: &StringScanner, mask: Mask64) -> Mask64 { mask & scanner.start_quote_and_string }
    /// Return a mask of whether the given characters are inside a string (only works on non-quotes)
    #[inline(always)]
    pub fn non_quote_outside_string(&self, scanner: &StringScanner, mask: Mask64) -> Mask64 { mask & !scanner.start_quote_and_string }
}

