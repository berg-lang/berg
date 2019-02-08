use crate::syntax::identifiers::*;
use crate::syntax::{AstIndex, AstRef, ByteIndex, ByteRange, Token};
use std::cmp;
use std::fmt;
use std::io;
use std::io::Read;

pub struct SourceReconstruction<'p, 'a: 'p> {
    ast: &'p AstRef<'a>,
    range: ByteRange,
}

pub struct SourceReconstructionReader<'p, 'a: 'p> {
    iterator: SourceReconstructionIterator<'p, 'a>,
    buffered: Option<&'p [u8]>,
}

/// Iterates through tokens and space, yielding &str's that reconstruct the file.
struct SourceReconstructionIterator<'p, 'a: 'p> {
    ast: &'p AstRef<'a>,
    index: ByteIndex,
    end: ByteIndex,
    ast_index: AstIndex,
    whitespace_indices: Vec<usize>,
    line_index: usize,
}

impl<'p, 'a: 'p> SourceReconstruction<'p, 'a> {
    pub fn new(ast: &'p AstRef<'a>, range: ByteRange) -> Self {
        SourceReconstruction { ast, range }
    }
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        // Set up the buffer.
        let size = usize::from(self.range.end - self.range.start);
        let mut buffer = Vec::with_capacity(size);
        unsafe { buffer.set_len(size) };

        // Read!
        let mut reader = SourceReconstructionReader::new(self.ast, self.range.clone());
        let size = reader.read(buffer.as_mut_slice()).unwrap();
        assert!(size == buffer.len());
        buffer
    }
}

impl<'p, 'a: 'p> fmt::Display for SourceReconstruction<'p, 'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let iterator = SourceReconstructionIterator::new(self.ast, self.range.clone());
        for bytes in iterator {
            write!(f, "{}", String::from_utf8_lossy(bytes))?;
        }
        Ok(())
    }
}

impl<'p, 'a: 'p> SourceReconstructionReader<'p, 'a> {
    pub fn new(ast: &'p AstRef<'a>, range: ByteRange) -> Self {
        SourceReconstructionReader {
            iterator: SourceReconstructionIterator::new(ast, range),
            buffered: None,
        }
    }

    fn next(&mut self) -> Option<&'p [u8]> {
        if let Some(buffer) = self.buffered.take() {
            Some(buffer)
        } else {
            self.iterator.next()
        }
    }
}

impl<'p, 'a: 'p> io::Read for SourceReconstructionReader<'p, 'a> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut read = 0;
        while let Some(mut bytes) = self.next() {
            assert!(!bytes.is_empty());

            read += bytes.read(&mut buffer[read..])?;
            if !bytes.is_empty() {
                // If we filled the buffer, stash it and return.
                self.buffered = Some(bytes);
                break;
            }
        }
        Ok(read)
    }
}

impl<'p, 'a: 'p> SourceReconstructionIterator<'p, 'a> {
    fn new(ast: &'p AstRef<'a>, range: ByteRange) -> Self {
        assert!(ast.tokens().len() > 0);
        let index = range.start;
        SourceReconstructionIterator {
            ast,
            index,
            end: range.end,
            ast_index: find_ast_index(ast, index),
            whitespace_indices: find_whitespace_indices(ast, index),
            line_index: 0,
        }
    }
}

impl<'p, 'a: 'p> Iterator for SourceReconstructionIterator<'p, 'a> {
    type Item = &'p [u8];
    fn next(&mut self) -> Option<&'p [u8]> {
        if self.index >= self.end {
            return None;
        }

        // Get the next thing (be it token or whitespace)
        let (start, mut bytes) = self
            .next_token()
            .or_else(|| self.next_whitespace_range())
            .or_else(|| self.next_newline())
            .unwrap_or_else(|| (self.index, SPACE.well_known_str().as_bytes()));

        // Clip the beginning of the string if it starts earlier than index.
        match start.cmp(&self.index) {
            cmp::Ordering::Equal => {}
            cmp::Ordering::Less => {
                bytes = &bytes[(self.index - start).into()..];
            }
            cmp::Ordering::Greater => unreachable!(),
        }
        // Clip the end of the string if it goes past the end.
        if self.index + bytes.len() > self.end {
            bytes = &bytes[..(self.end - self.index).into()]
        }
        // Increment the index, and return!
        self.index += bytes.len();
        Some(bytes)
    }
}

impl<'p, 'a: 'p> SourceReconstructionIterator<'p, 'a> {
    fn next_token(&mut self) -> Option<(ByteIndex, &'p [u8])> {
        let token_ranges = self.ast.token_ranges();
        let token_range = &token_ranges[self.ast_index];
        if self.index >= token_range.start && self.index < token_range.end {
            // Grab the string we are returning this time.
            let result = self.token_bytes(token_range.start, self.ast.tokens()[self.ast_index]);
            let end = match result {
                Some((start, bytes)) => start + bytes.len(),
                None => token_range.end,
            };

            // Skip to the next non-empty ast index now that we've passed this token.
            if end >= token_range.end {
                while self.ast_index + 1 < token_ranges.len() {
                    self.ast_index += 1;
                    if token_ranges[self.ast_index].start < token_ranges[self.ast_index].end {
                        break;
                    }
                }
            }

            // Return the string.
            result
        } else {
            None
        }
    }

    fn next_whitespace_range(&mut self) -> Option<(ByteIndex, &'p [u8])> {
        // Go through our catalogue of whitespace, and find out if any are in our range.
        let char_ranges = &self.ast.char_data().whitespace.char_ranges;
        for (index, &(ref space_char, ref ranges)) in char_ranges.iter().enumerate() {
            // If this space char's next range is at our index, return it.
            let range_index = self.whitespace_indices[index];
            let range = &ranges[range_index];
            if range.start <= self.index && self.index < range.end {
                // Figure out where this repeat of the space character starts (it could be a multibyte character).
                let offset = usize::from(self.index - range.start) % space_char.len();
                let start = self.index - offset;

                // Skip to the next whitespace if we need to.
                if start + space_char.len() >= range.end && range_index + 1 < ranges.len() {
                    self.whitespace_indices[index] = range_index + 1;
                }

                return Some((start, space_char.as_bytes()));
            }
        }
        None
    }

    fn next_newline(&mut self) -> Option<(ByteIndex, &'p [u8])> {
        let line_starts = &self.ast.char_data().line_starts;
        // If we are looking for the character just before the end of the line, it's \n.
        if self.line_index + 1 < line_starts.len()
            && self.index == line_starts[self.line_index + 1] - 1
        {
            self.line_index += 1;
            let string = NEWLINE.well_known_str().as_bytes();
            assert!(string.len() == 1);
            Some((self.index, string))
        } else {
            None
        }
    }

    fn token_bytes(&self, token_start: ByteIndex, token: Token) -> Option<(ByteIndex, &'p [u8])> {
        use crate::syntax::token::Token::*;
        let bytes = match token {
            IntegerLiteral(literal) | ErrorTerm(.., literal) => {
                self.ast.literal_string(literal).as_bytes()
            }
            RawErrorTerm(.., raw_literal) => &self.ast.raw_literals()[raw_literal],

            FieldReference(field) => {
                self.ast.identifier_string(self.ast.fields()[field].name).as_bytes()
            }

            RawIdentifier(identifier)
            | InfixOperator(identifier)
            | PostfixOperator(identifier)
            | PrefixOperator(identifier) => self.ast.identifier_string(identifier).as_bytes(),

            InfixAssignment(identifier) => {
                // Because of how InfixAssignment works, we store the str for the "+" and assume the "="
                let bytes = self.ast.identifier_string(identifier).as_bytes();
                if self.index == token_start + bytes.len() {
                    return Some((token_start + bytes.len(), b"="));
                } else {
                    bytes
                }
            }

            Open { boundary, .. } => boundary.open_string().as_bytes(),
            OpenBlock { index, .. } => self.ast.blocks()[index].boundary.open_string().as_bytes(),
            Close { boundary, .. } => boundary.close_string().as_bytes(),
            CloseBlock { index, .. } => self.ast.blocks()[index].boundary.close_string().as_bytes(),
            NewlineSequence => return None,
            MissingExpression | Apply => unreachable!(),
        };
        Some((token_start, bytes))
    }
}

fn find_ast_index(ast: &AstRef, index: ByteIndex) -> AstIndex {
    let ast_index = ast
        .token_ranges()
        .iter()
        .position(|range| range.end > index);
    ast_index.unwrap_or_else(|| ast.token_ranges().last_index())
}

fn find_whitespace_indices(ast: &AstRef, index: ByteIndex) -> Vec<usize> {
    ast.char_data()
        .whitespace
        .char_ranges
        .iter()
        .map(|&(_, ref ranges)| {
            let whitespace_index = ranges.iter().position(|range| range.end > index);
            whitespace_index.unwrap_or_else(|| ranges.len() - 1)
        })
        .collect()
}
