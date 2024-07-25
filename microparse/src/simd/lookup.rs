use std::simd::{cmp::SimdPartialEq, Simd};

// TODO parameterize this instead
use crate::arch::native::{SimdU8, splat16, lookup_lower16_ascii};

pub struct LookupLower16(SimdU8);

pub struct MatchLower16(LookupLower16);

impl LookupLower16 {
    pub const fn new<const N: usize>(mappings: [(u8, u8); N], default: u8) -> Self {
        Self::new_from_subslice(&mappings, N, default)
    }

    const fn new_from_subslice<const N: usize>(mappings: &[(u8, u8); N], len: usize, default: u8) -> Self {
        assert!(N <= 16, "Too many matches! Nust be 16 or less.");
        assert!(len < N);

        let mut table = [default; 16];
        let mut filled = [false; 16];

        // We have to unroll the loop manually with this macro, because we're in a const fn and can't
        // loop normally.
        seq_macro::seq!(i in 0..16 {
            if i < len {
                let lower16 = (mappings[i].0 & 0b0000_1111) as usize;

                assert!((mappings[i].0 & 0b1000_0000) == 0, "Match character is not ASCII! Lookup will not work on Intel platforms.");
                assert!(!filled[lower16], "Multiple match characters with the same lower 16 bytes!");

                table[lower16] = mappings[i].1;
                filled[lower16] = true;
            }
        });

        Self(splat16(Simd::from_array(table)))
    }

    #[inline(always)]
    pub fn lookup(&self, keys: SimdU8) -> SimdU8 {
        lookup_lower16_ascii(keys, self.0)
    }
}

impl MatchLower16 {
    pub const fn new<const N: usize>(matches: [u8; N]) -> Self {
        if N > 16 {
            panic!("Too many matches! Must be 16 or less.");
        }

        let mut mappings = [(0u8, 0u8); 16];
        let mut found0 = false;
        seq_macro::seq!(i in 0..16 {
            if i < N {
                let lower16 = matches[i] & 0b0000_1111;
                found0 = found0 || lower16 == 0;
                mappings[i] = (matches[i], matches[i]);
            }
        });

        // We don't want 0 to match unless the user passed in something that resolves to 0, so we
        // add it manually
        if found0 {
            Self(LookupLower16::new_from_subslice(&mappings, matches.len(), 0))
        } else {
            mappings[matches.len()] = (0, 1);
            Self(LookupLower16::new_from_subslice(&mappings, matches.len()+1, 0))
        }
    }

    #[inline(always)]
    pub fn matches(&self, chars_to_match: SimdU8) -> <SimdU8 as SimdPartialEq>::Mask {
        self.0.lookup(chars_to_match).simd_eq(chars_to_match)
    }
}