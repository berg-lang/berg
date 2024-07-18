use super::{Mask64, BLOCK_SIZE, ODD_BITS};

///
/// Scan for escaped characters preceded by an escape (like backslash).
/// 
/// Takes particular care to ensure that runs of escapes are handled correctly:
/// - `"abc\\\","` is a string with a quote and a comma in it.
/// - `"abc\\","` is a string followed by a comma and the start of another string.
/// 
pub struct EscapeScanner<const SHORT_CIRCUIT_NO_BACKSLASHES: bool = true> {
    /// Whether the first character of the next block is escaped.
    pub next_is_escaped: Mask64,
}

pub struct Escapes {
    ///
    /// Mask of escaped characters.
    ///
    /// ```
    /// \n \\n \\\n \\\\n \
    /// 0100100010100101000
    ///  n  \   \ n  \ \
    /// ```
    ///
    pub escaped: Mask64,
    ///
    /// Mask of escape characters.
    ///
    /// ```
    /// \n \\n \\\n \\\\n \
    /// 1001000101001010001
    /// \  \   \ \  \ \   \
    /// ```
    ///
    pub escape: Mask64,
}

impl<const SHORT_CIRCUIT_NO_BACKSLASHES: bool> Default for EscapeScanner<SHORT_CIRCUIT_NO_BACKSLASHES> {
    #[inline(always)]
    fn default() -> Self {
        Self { next_is_escaped: 0 }
    }
}

impl<const SHORT_CIRCUIT_NO_BACKSLASHES: bool> EscapeScanner<SHORT_CIRCUIT_NO_BACKSLASHES> {
    ///
    /// Get a mask of both escape and escaped characters (the characters following a backslash).
    ///
    /// @param backslash A mask of the character that can escape others (but could be
    ///        escaped itself). e.g. block.eq('\\')
    #[inline(always)]
    pub fn next(&mut self, backslash: Mask64) -> Escapes {
        if SHORT_CIRCUIT_NO_BACKSLASHES {
            return Escapes { escaped: self.next_escaped_without_backslashes(), escape: 0 }
        }
    
        // |                                | Mask (shows characters instead of 1's) | Depth | Instructions        |
        // |--------------------------------|----------------------------------------|-------|---------------------|
        // | string                         | `\\n_\\\n___\\\n___\\\\___\\\\__\\\`   |       |                     |
        // |                                | `    even   odd    even   odd   odd`   |       |                     |
        // | potential_escape               | ` \  \\\    \\\    \\\\   \\\\  \\\`   | 1     | 1 (backslash & ~first_is_escaped)
        // | escape_and_terminal_code       | ` \n \ \n   \ \n   \ \    \ \   \ \`   | 5     | 5 (next_escape_and_terminal_code())
        // | escaped                        | `\    \ n    \ n    \ \    \ \   \ ` X | 6     | 7 (escape_and_terminal_code ^ (potential_escape | first_is_escaped))
        // | escape                         | `    \ \    \ \    \ \    \ \   \ \`   | 6     | 8 (escape_and_terminal_code & backslash)
        // | first_is_escaped               | `\                                 `   | 7 (*) | 9 (escape >> 63) ()
        //                                                                               (*) this is not needed until the next iteration
        let escape_and_terminal_code = Self::next_escape_and_terminal_code(backslash & !self.next_is_escaped);
        let escaped = escape_and_terminal_code ^ (backslash | self.next_is_escaped);
        let escape = escape_and_terminal_code & backslash;
        // We do this now instead of when it's used so that the fast path doesn't have to do it on every iteration.
        self.next_is_escaped = escape >> (BLOCK_SIZE-1);
        Escapes { escaped, escape }
    }

    ///
    /// Gets the list of escaped characters when there are no backslashes in the current block.
    /// 
    /// Just takes previous block's overflow (next_is_escaped) and returns it.
    /// 
    fn next_escaped_without_backslashes(&mut self) -> Mask64 {
        let escaped = self.next_is_escaped;
        self.next_is_escaped = 0;
        escaped
    }
    
    ///
    /// Returns a mask of the next escape characters (masking out escaped backslashes), along with
    /// any non-backslash escape codes.
    ///
    /// \n \\n \\\n \\\\n returns:
    /// \n \   \ \n \ \
    /// 11 100 1011 10100
    ///
    /// You are expected to mask out the first bit yourself if the previous block had a trailing
    /// escape.
    ///
    /// & the result with potential_escape to get just the escape characters.
    /// ^ the result with (potential_escape | first_is_escaped) to get escaped characters.
    ///
    fn next_escape_and_terminal_code(potential_escape: Mask64) -> Mask64 {
        // If we were to just shift and mask out any odd bits, we'd actually get a *half* right answer:
        // any even-aligned backslash runs would be correct! Odd-aligned backslash runs would be
        // inverted (\\\ would be 010 instead of 101).
        //
        // ```
        // string:              | ____\\\\_\\\\_____ |
        // maybe_escaped | ODD  |     \ \   \ \      |
        //               even-aligned ^^^  ^^^^ odd-aligned
        // ```
        //
        // Taking that into account, our basic strategy is:
        //
        // 1. Use subtraction to produce a mask with 1's for even-aligned runs and 0's for
        //    odd-aligned runs.
        // 2. XOR all odd bits, which masks out the odd bits in even-aligned runs, and brings IN the
        //    odd bits in odd-aligned runs.
        // 3. & with backslash to clean up any stray bits.
        // runs are set to 0, and then XORing with "odd":
        //
        // |                                | Mask (shows characters instead of 1's) | Instructions        |
        // |--------------------------------|----------------------------------------|---------------------|
        // | string                         | `\\n_\\\n___\\\n___\\\\___\\\\__\\\`   |
        // |                                | `    even   odd    even   odd   odd`   |
        // | maybe_escaped                  | `  n  \\n    \\n    \\\_   \\\_  \\` X | 1 (potential_escape << 1)
        // | maybe_escaped_and_odd          | ` \n_ \\n _ \\\n_ _ \\\__ _\\\_ \\\`   | 1 (maybe_escaped | odd)
        // | even_series_codes_and_odd      | `  n_\\\  _    n_ _\\\\ _     _    `   | 1 (maybe_escaped_and_odd - potential_escape)
        // | escape_and_terminal_code       | ` \n \ \n   \ \n   \ \    \ \   \ \`   | 1 (^ odd)
        //
    
        // Escaped characters are characters following an escape.
        let maybe_escaped = potential_escape << 1;
    
        // To distinguish odd from even escape sequences, therefore, we turn on any *starting*
        // escapes that are on an odd byte. (We actually bring in all odd bits, for speed.)
        // - Odd runs of backslashes are 0000, and the code at the end ("n" in \n or \\n) is 1.
        // - Odd runs of backslashes are 1111, and the code at the end ("n" in \n or \\n) is 0.
        // - All other odd bytes are 1, and even bytes are 0.
        let maybe_escaped_and_odd_bits     = maybe_escaped | ODD_BITS;
        let even_series_codes_and_odd_bits = maybe_escaped_and_odd_bits - potential_escape;
    
        // Now we flip all odd bytes back with xor. This:
        // - Makes odd runs of backslashes go from 0000 to 1010
        // - Makes even runs of backslashes go from 1111 to 1010
        // - Sets actually-escaped codes to 1 (the n in \n and \\n: \n = 11, \\n = 100)
        // - Resets all other bytes to 0
        even_series_codes_and_odd_bits ^ ODD_BITS
    }
}

