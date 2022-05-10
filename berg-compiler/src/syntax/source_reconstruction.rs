use crate::syntax::{Ast, AstIndex, ByteIndex, ByteRange};
use std::borrow::Cow;
use std::fmt;
use std::io;
use std::io::Read;

///
/// Reconstructs a range of source from the parsed AST.
///
/// All data in an AST is preserved, which is what makes this possible.
///
pub struct SourceReconstruction<'p, 'a: 'p> {
    ast: &'p Ast<'a>,
    range: ByteRange,
}

///
/// An io::Reader over an AST that yields the same data as the original source.
///
/// Uses [`SourceReconstruction`] to do the formatting.
///
pub struct SourceReconstructionReader<'p, 'a: 'p> {
    iterator: SourceReconstructionIterator<'p, 'a>,
    buffered: Option<Cow<'p, [u8]>>,
}

///
/// Iterates through the AST, yielding &strs that reconstruct the file.
///
/// Works by iterating in parallel through tokens, whitespace and line starts,
/// and picking whichever one covers the current range.
///
struct SourceReconstructionIterator<'p, 'a: 'p> {
    /// The AST we're reconstructing.
    ast: &'p Ast<'a>,
    /// The current byte index (corresponding to the original file).
    index: ByteIndex,
    /// The end of the range we're reconstructing (non-inclusive)
    end: ByteIndex,
    /// The next token.
    ast_index: AstIndex,
    /// The next comment.
    comment_index: usize,
    /// The next whitespace.
    whitespace_index: usize,
    /// The next line start.
    line_start_index: usize,
}

impl<'p, 'a: 'p> SourceReconstruction<'p, 'a> {
    ///
    /// Create a reconstructor for the given range.
    ///
    /// # Arguments
    ///
    /// * `ast` - The AST containing the parsed information
    /// *
    ///
    pub fn new(ast: &'p Ast<'a>, range: ByteRange) -> Self {
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
            write!(f, "{}", String::from_utf8_lossy(&bytes))?;
        }
        Ok(())
    }
}

impl<'p, 'a: 'p> SourceReconstructionReader<'p, 'a> {
    pub fn new(ast: &'p Ast<'a>, range: ByteRange) -> Self {
        SourceReconstructionReader {
            iterator: SourceReconstructionIterator::new(ast, range),
            buffered: None,
        }
    }

    fn next(&mut self) -> Option<Cow<'p, [u8]>> {
        self.buffered.take().or_else(|| self.iterator.next())
    }
}

impl<'p, 'a: 'p> io::Read for SourceReconstructionReader<'p, 'a> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut read = 0;
        while let Some(mut bytes) = self.next() {
            assert!(!bytes.is_empty());
            read += match bytes {
                Cow::Borrowed(ref mut bytes) => bytes.read(&mut buffer[read..])?,
                Cow::Owned(ref mut vec) => {
                    let read = (&mut vec.as_slice()).read(&mut buffer[read..])?;
                    vec.drain(0..read);
                    read
                }
            };
            if !bytes.is_empty() {
                // If we have leftover after filling the buffer, stash it and return.
                self.buffered = Some(bytes);
                break;
            }
        }
        Ok(read)
    }
}

impl<'p, 'a: 'p> SourceReconstructionIterator<'p, 'a> {
    fn new(ast: &'p Ast<'a>, range: ByteRange) -> Self {
        assert!(ast.tokens.len() > 0);
        let index = range.start;
        SourceReconstructionIterator {
            ast,
            index,
            end: range.end,
            ast_index: find_ast_index(ast, index),
            comment_index: find_comment_index(ast, index),
            whitespace_index: find_whitespace_index(ast, index),
            line_start_index: 0,
        }
    }
}

impl<'p, 'a: 'p> Iterator for SourceReconstructionIterator<'p, 'a> {
    type Item = Cow<'p, [u8]>;
    fn next(&mut self) -> Option<Cow<'p, [u8]>> {
        if self.index >= self.end {
            return None;
        }

        // Get the next thing (be it token or whitespace)
        let bytes = self
            .next_token()
            .or_else(|| self.next_whitespace_range())
            .or_else(|| self.next_comment())
            .or_else(|| self.next_newline())
            .unwrap_or_else(|| Cow::Borrowed(b" "));

        // Increment the index, and return!
        self.index += bytes.len();
        Some(bytes)
    }
}

impl<'p, 'a: 'p> SourceReconstructionIterator<'p, 'a> {
    fn next_token(&mut self) -> Option<Cow<'p, [u8]>> {
        while let Some(ByteRange { start, end }) = self.ast.token_ranges.get(self.ast_index) {
            // If the current token isn't ready to emit yet, return.
            if *start > self.index {
                break;
            }

            // Return the token string
            self.ast_index += 1;
            // Skip empty tokens
            if *end > *start {
                let token = self.ast.tokens[self.ast_index - 1];
                return self.truncate(*start, token.original_bytes(self.ast));
            }
        }
        None
    }

    fn next_comment(&mut self) -> Option<Cow<'p, [u8]>> {
        if let Some((comment, comment_start)) = self.ast.char_data.comments.get(self.comment_index)
        {
            if *comment_start <= self.index {
                self.comment_index += 1;
                assert!(
                    *comment_start + comment.len() > self.index,
                    "comment {:?} at {} got skipped somehow! Current index is {}.",
                    comment,
                    comment_start,
                    self.index
                );
                return self.truncate(*comment_start, comment);
            }
        }

        None
    }

    fn truncate(&self, start: ByteIndex, bytes: impl Into<Cow<'p, [u8]>>) -> Option<Cow<'p, [u8]>> {
        let mut bytes = bytes.into();
        // Clip the beginning of the string if it starts earlier than index.
        assert!(start <= self.index);
        assert!(!bytes.is_empty());
        if start < self.index {
            bytes = match bytes {
                Cow::Borrowed(bytes) => bytes[(self.index - start).into()..].into(),
                Cow::Owned(mut vec) => {
                    vec.drain(0..(self.index - start).into());
                    vec.into()
                }
            };
        }
        assert!(!bytes.is_empty());
        // Clip the end of the string if it goes past the end of the region we're printing.
        let len = usize::from(self.end - self.index);
        if bytes.len() > len {
            bytes = match bytes {
                Cow::Borrowed(bytes) => bytes[..len].into(),
                Cow::Owned(mut vec) => {
                    vec.truncate(len);
                    vec.into()
                }
            };
        }
        Some(bytes)
    }

    fn next_whitespace_range(&mut self) -> Option<Cow<'p, [u8]>> {
        // If the current whitespace range includes us, return that string.
        if let Some((whitespace, whitespace_start)) = self
            .ast
            .char_data
            .whitespace_ranges
            .get(self.whitespace_index)
        {
            if *whitespace_start <= self.index {
                self.whitespace_index += 1;
                let whitespace_string = self.ast.whitespace_string(*whitespace);
                assert!(
                    *whitespace_start + whitespace_string.len() > self.index,
                    "whitespace {:?} at {} got skipped somehow! Current index is {}.",
                    whitespace_string,
                    whitespace_start,
                    self.index
                );
                return self.truncate(*whitespace_start, whitespace_string.as_bytes());
            }
        }

        None
    }

    ///
    /// Write out \n if we have a line ending without any character data.
    ///
    fn next_newline(&mut self) -> Option<Cow<'p, [u8]>> {
        // If this is a line start, increment the line start index and return "\n".
        while let Some(line_start) = self.ast.char_data.line_starts.get(self.line_start_index) {
            // We haven't reached the line ending yet.
            if *line_start > self.index + 1 {
                break;
            }

            // We may have to skip a few line starts if some of them had alternate
            // line endings like \r or \r\n in them (and therefore we found the space
            // string in next_whitespace()).
            self.line_start_index += 1;
            if *line_start == self.index + 1 {
                return Some("\n".as_bytes().into());
            }
        }

        None
    }
}

fn find_ast_index(ast: &Ast, index: ByteIndex) -> AstIndex {
    // Get the first token that ends after the index and is non-empty
    let ast_index = ast
        .token_ranges
        .iter()
        .position(|range| range.end > index && range.end > range.start);
    ast_index.unwrap_or_else(|| ast.token_ranges.len().into())
}

fn find_comment_index(ast: &Ast, index: ByteIndex) -> usize {
    // Get the first comment that ends after the index
    let comment_index = ast
        .char_data
        .comments
        .iter()
        .position(|(_, start)| *start >= index);
    comment_index.unwrap_or_else(|| ast.char_data.comments.len())
}

fn find_whitespace_index(ast: &Ast, index: ByteIndex) -> usize {
    // Get the first whitespace that starts *at* or *after* the given index.
    if let Some(next_whitespace) = ast
        .char_data
        .whitespace_ranges
        .iter()
        .position(|(_, start)| *start >= index)
    {
        // If there is a whitespace starting *after* the given index, check if the previous one *intersects* the index.
        if next_whitespace > 0 {
            let (whitespace, start) = ast.char_data.whitespace_ranges[next_whitespace - 1];
            if start
                + ast
                    .char_data
                    .whitespace_characters
                    .resolve(whitespace)
                    .unwrap()
                    .len()
                > index
            {
                return next_whitespace - 1;
            }
        }
        return next_whitespace;
    }
    ast.char_data.whitespace_ranges.len()
}
