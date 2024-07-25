mod classifier;

use std::{iter::Scan, simd::cmp::SimdPartialEq as _};
use crate::{
    arch::native::*,
    primitive_mask::{
        escape::EscapeScanner,
        preceded_by::PrecededByScanner,
        string::{StringScanner, Strings},
        test::InputMask,
        Mask64
    },
    simd::{chunks_padded::SimdChunksPadded, lookup::MatchLower16}, unwrapping_iterator::IntoUnwrappingIterator as _,
};

#[derive(Default)]
pub struct Parser {
    structural_indexes: Vec<u8>,
    escape_scanner: EscapeScanner,
    string_scanner: StringScanner,
    follows_nonquote_scalar: PrecededByScanner,
}

struct Utf8Validator {
}

struct Classifier {

}
/// Classifies SIMD-sized chunks of bytes, appending them to 64- or SIMD-sized bitmasks.
struct Classifier {
    raw_quote: Bitmask64Builder,
    backslash: Bitmask64Builder,
    whitespace: SimdBitmaskBuilder,
    operators: SimdBitmaskBuilder,
}

impl Classifier {
    #[inline(always)]
    fn classify(self, bytes: &[u8; SIMD_BITS], chunk64_index: usize, simd_index: usize) {
        let simd = SimdU8::from_slice(bytes[(64*chunk64_index) + (SIMD_BYTES*simd_index)]);
        
    }
}

struct 
impl Parser {
    #[inline(always)]
    pub fn parse(&mut self, input: &[u8]) {
        // For each simd bitmask (128, 256, 512 bytes) worth of bytes ...
        let simd_bits_chunks = input.array_chunks::<SIMD_BITS>();
        for simd_bits_chunk in simd_bits_chunks {
            // For each 64-byte chunk (for 64-bit bitmasks) ... (64-bit operations--escape+string)
            crate::fold_n::fold_n::<SIMD64_LANES>((), |(), index64| {
                // For each SIMD-sized byte chunk inside our 64-byte chunk ... (classify bytes)
                crate::fold_n::fold_n::<{ 64 / SIMD_BYTES }>((), |(), simd_index| {
                    
                })
            });
        }

        let simd_bits_chunk = simd_bits_chunks.remainder().array_chunks::<SIMD_PER_64_BYTES>();
        for chunk64_byte_chunks in simd_bits_chunk {
            let mask64 = as_64_byte_chunks(chunk64_byte_chunks).into_unwrapping_iter().fold((), |(), simd_bytes, index| {
                let simd = SimdU8::from_array(*simd_bytes);
            });
            let mask64 = chunk64_byte_chunks.into_unwrapping_iter().fold((), |(), simd_bytes, index| {
                let simd = SimdU8::from_array(*simd_bytes);
            });
        }
        while remainder.len() >= SIMD_PER_64_BYTES {


        }
        while remainder.len() > SIMD_BYTES {

        }
        if !chunks.remainder().is_empty() {
            while 
            let simd64chunks = chunks.remainder().array_chunks();
            for chunk in simd64chunks {
                self.parse_chunk(SimdU8::from_array(*chunk));
            }
        }
    }
    
    fn classify_bytes(&mut self, input: SimdU8) -> ClassifiedBytes {
        
    }

    // fn classify(&mut self, input: SimdU8) -> ClassifiedBytes {

    // }

    // #[inline(always)]
    // fn parse_chunk(&mut self, input: SimdU8) {
    //     let raw_quote = input.simd_eq(SimdU8::splat(b'"')).to_bitmask();
    //     let backslash = input.simd_eq(SimdU8::splat(b'\\')).to_bitmask();
    //     let strings = {
    //         let escaped = self.parse_escaped(input);
    //         self.parse_strings(quote, escaped);
    //     }
    //     let escaped = 
    //     let strings = self.parse_strings(input);

    //     // The term "scalar" refers to anything except structural characters and white space
    //     // (so letters, numbers, quotes).
    //     // We want follows_scalar to mark anything that follows a non-quote scalar (so letters and numbers).
    //     //
    //     // A terminal quote should either be followed by a structural character (comma, brace, bracket, colon)
    //     // or nothing. However, we still want ' "a string"true ' to mark the 't' of 'true' as a potential
    //     // pseudo-structural character just like we would if we had  ' "a string" true '; otherwise we
    //     // may need to add an extra check when parsing strings.
    //     //
    //     // Performance: there are many ways to skin this cat.
    //     let (operators, scalar) = self.parse_scalars(input);
    //     let nonquote_scalar = scalar & !strings.quote;
    //     // _follows_potential_nonquote_scalar: is defined as marking any character that follows a character
    //     // that is not a structural element ({,},[,],:, comma) nor a quote (") and that is not a
    //     // white space.
    //     // It is understood that within quoted region, anything at all could be marked (irrelevant).
    //     let follows_potential_nonquote_scalar = self.follows_nonquote_scalar.next(nonquote_scalar);
    //     // TODO now index them!
    //     index_them();

    //     // The term "scalar" refers to anything except structural characters and white space
    //     // (so letters, numbers, quotes).
    //     // Whenever it is preceded by something that is not a structural element ({,},[,],:, ") nor a white-space
    //     // then we know that it is irrelevant structurally.
    //         let potential_scalar_start = scalar & follows_potential_nonquote_scalar;

    //     let potential_structural_start = operators | potential_scalar_start;
    //     let end_quote_and_string = strings.end_quote_and_string(&self.string_scanner);
    //     let structural_start = potential_structural_start & !end_quote_and_string;
    //     prev_structurals = block.structural_start();
    //     self.unescaped_chars_error |= strings.non_quote_inside_string(unescaped);
    // }

    // #[inline(always)]
    // fn parse_scalars(&mut self, input: SimdU8) -> (Mask64, Mask64) {
    //     let scalar = self.parse_scalars(input);
    //     let whitespace = WHITESPACE.matches(input).to_bitmask();
    //     // We twiddle the input to turn [ into { and ] into }.
    //     let curlified = input | SimdU8::splat(0x20);
    //     let operators = OPERATORS.matches(curlified).to_bitmask();
    //     let scalar = !(operators | whitespace);
    //     (operators, scalar)
    // }

    // #[inline(always)]
    // fn parse_strings(&mut self, input: SimdU8) -> Strings {
    //     let backslash = input.simd_eq(SimdU8::splat(b'\\')).to_bitmask();
    //     let raw_quote = input.simd_eq(SimdU8::splat(b'"')).to_bitmask();
    //     let quote = raw_quote & !self.escape_scanner.next(backslash).escaped;
    //     self.string_scanner.next(quote)
    // }
}