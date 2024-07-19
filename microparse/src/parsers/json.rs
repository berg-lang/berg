use std::simd::cmp::SimdPartialEq as _;

use crate::{arch::native::*, primitive_mask::{escape::EscapeScanner, preceded_by::PrecededByScanner, string::{StringScanner, Strings}}, simd::{chunks_padded::SimdChunksPadded, lookup::MatchLower16}};

#[derive(Default)]
pub struct Parser {
    structural_indexes: Vec<u8>,
    escape_scanner: EscapeScanner,
    string_scanner: StringScanner,
    follows_nonquote_scalar: PrecededByScanner,
}

const WHITESPACE: MatchLower16 = MatchLower16::new([b' ', b'\t', b'\n', b'\r']);
const OPERATORS: MatchLower16 = MatchLower16::new([b':', b'{', b',', b'}']);

struct WhitespaceAndStructurals {
    whitespace: u64,
    structurals: u64,
}

impl Parser {
    #[inline(always)]
    pub fn parse(&mut self, input: &[u8]) {
        input.simd_chunks_padded(b' ').for_each(|chunk| self.parse_chunk(chunk));
    }

    #[inline(always)]
    fn parse_chunk(&mut self, input: SimdU8) {
        let strings = self.parse_strings(input);

        // The term "scalar" refers to anything except structural characters and white space
        // (so letters, numbers, quotes).
        // We want follows_scalar to mark anything that follows a non-quote scalar (so letters and numbers).
        //
        // A terminal quote should either be followed by a structural character (comma, brace, bracket, colon)
        // or nothing. However, we still want ' "a string"true ' to mark the 't' of 'true' as a potential
        // pseudo-structural character just like we would if we had  ' "a string" true '; otherwise we
        // may need to add an extra check when parsing strings.
        //
        // Performance: there are many ways to skin this cat.
        let (operators, scalar) = self.parse_scalars(input);
        let nonquote_scalar = scalar & !strings.quote;
        // _follows_potential_nonquote_scalar: is defined as marking any character that follows a character
        // that is not a structural element ({,},[,],:, comma) nor a quote (") and that is not a
        // white space.
        // It is understood that within quoted region, anything at all could be marked (irrelevant).
        let follows_potential_nonquote_scalar = self.follows_nonquote_scalar.next(nonquote_scalar);
        // TODO now index them!
        index_them();

        // The term "scalar" refers to anything except structural characters and white space
        // (so letters, numbers, quotes).
        // Whenever it is preceded by something that is not a structural element ({,},[,],:, ") nor a white-space
        // then we know that it is irrelevant structurally.
            let potential_scalar_start = scalar & follows_potential_nonquote_scalar;

        let potential_structural_start = operators | potential_scalar_start;
        let end_quote_and_string = strings.end_quote_and_string(&self.string_scanner);
        let structural_start = potential_structural_start & !end_quote_and_string;
        prev_structurals = block.structural_start();
        self.unescaped_chars_error |= strings.non_quote_inside_string(unescaped);
    }

    #[inline(always)]
    fn parse_scalars(&mut self, input: SimdU8) -> (Mask64, Mask64) {
        let scalar = self.parse_scalars(input);
        let whitespace = WHITESPACE.matches(input).to_bitmask();
        // We twiddle the input to turn [ into { and ] into }.
        let curlified = input | SimdU8::splat(0x20);
        let operators = OPERATORS.matches(curlified).to_bitmask();
        let scalar = !(operators | whitespace);
        (operators, scalar)
    }

    #[inline(always)]
    fn parse_strings(&mut self, input: SimdU8) -> Strings {
        let backslash = input.simd_eq(SimdU8::splat(b'\\')).to_bitmask();
        let raw_quote = input.simd_eq(SimdU8::splat(b'"')).to_bitmask();
        let quote = raw_quote & !self.escape_scanner.next(backslash).escaped;
        self.string_scanner.next(quote)
    }
}