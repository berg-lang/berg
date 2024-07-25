// Classifies and validates a single SIMD-sized chunk of bytes.

use crate::{primitive_mask::Mask64, simd::lookup::MatchLower16};

use super::SimdU8;

struct Classifier {
}

struct Classified {
    backslash: Mask64,
    raw_quote: Mask64,
    whitespace: Mask64,
    operator: Mask64,
    unescaped_control_character: Mask64,
}

const WHITESPACE: MatchLower16 = MatchLower16::new([b' ', b'\t', b'\n', b'\r']);
const OPERATORS: MatchLower16 = MatchLower16::new([b':', b'{', b',', b'}']);

impl Classifier {
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
}