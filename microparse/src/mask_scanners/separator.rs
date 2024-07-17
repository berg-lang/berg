use super::Mask64;

///
/// Scans for separators, breaking up regions of other characters.
/// 
/// You specify two character classes: separators and value characters. After using this operation,
/// you can determine the start of each region of value characters, and the separator that ends
/// that region.
/// 
/// For example, if "," is a separator and identifiers are value characters, and whitespace is
/// neither (thus ignored), this will help you distinguish the start of each identifier and
/// the separator:
/// 
/// ```text
///  abc, def, ghi
///  |  | |  | |
/// ```
/// 
/// Because the value region can include whitespace anywhere (except the first character, which is
/// always a value character), this will not tell you if your value is broken up by whitespace, nor
/// will separators always immediately follow the value (so you can't use it to tell you the exact
/// end of the value without more work). To wit:
/// 
/// ```text
///  abc def, ghi jkl , mno
///  |      | |       | |
/// ```
/// 
/// Finally, if there are *multiple* separators, you can see the "extra" ones. Separators at the
/// beginning of a document (before any values) also show up as "extra."
/// 
/// Fundamentally, this starts in a "non-value region," switches to a "value region" when it detects
/// the first value character, and switches back to "non-value region" when it detects a separator.
/// It marks the first character in each region (the first value or separator character the flipped
/// the region) differently than others.
/// 
///  SS VVVVS VV VV S S V
/// 011010000010100100101
pub struct SeparatedValuesScanner {
    pub still_in_value_region: bool,
}

pub struct SeparatedValues {
    /// Mask of separator characters.
    pub separator_character: Mask64,
    /// Mask of value characters.
    pub value: Mask64,
    /// Difference.
    /// - difference & value: starting value character for each value region.
    /// - difference & ~value: all other value characters in value regions.
    /// - difference & separator: "extra" separators.
    /// - difference & ~separator: separators that end value regions.
    pub difference: Mask64,
}

impl SeparatedValuesScanner {
    /// Find regions of value characters separated by separator characters.
    #[inline]
    pub fn next(&mut self, separator_character: Mask64, value_character: Mask64) -> SeparatedValues {
        let (difference, still_in_value_region) = separator_character.borrowing_sub(value_character, self.still_in_value_region);
        self.still_in_value_region = still_in_value_region;
        SeparatedValues { separator_character, value: value_character, difference }
    }
}

impl SeparatedValues {
    #[inline]
    pub fn value_start(&self) -> Mask64 { self.difference & self.value }
    #[inline]
    pub fn value_tail(&self) -> Mask64 { self.difference & !self.value }
    #[inline]
    pub fn separator(&self) -> Mask64 { self.difference & !self.separator_character }
    #[inline]
    pub fn extra_separator(&self) -> Mask64 { self.difference & self.separator_character }
    #[inline]
    pub fn ignored(&self) -> Mask64 { !self.value & !self.separator_character}
    #[inline]
    pub fn ignored_in_value_region(&self) -> Mask64 { self.difference & self.ignored() }
    #[inline]
    pub fn ignored_before_value(&self) -> Mask64 { self.difference & !self.ignored() }
}