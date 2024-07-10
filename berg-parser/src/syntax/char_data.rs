use berg_util::Delta;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::u32;
use string_interner::backend::StringBackend;
use string_interner::{StringInterner, Symbol};

use crate::bytes::{line_column::DocumentLines, ByteIndex};

use super::ast::AstIndex;

///
/// Debug data about the original source.
///
/// Includes enough character data to reconstruct the original source.
///
#[derive(Debug)]
pub struct CharData {
    //
    // Line / column data for the document (including its size).
    //
    pub lines: DocumentLines,

    ///
    /// Whitespace characters found in the document.
    ///
    pub whitespace_characters: StringInterner<StringBackend<WhitespaceIndex>>,

    ///
    /// Ordered list of whitespace ranges, except ' ' and '\n'
    /// (which are the default for space and line ranges, respectively).
    ///
    /// Only the starting byte index is stored, since the length is stored in
    /// [`whitespace_characters`].
    ///
    pub whitespace_ranges: Vec<(WhitespaceIndex, ByteIndex)>,

    ///
    /// Ordered list of comments in the document.
    ///
    /// These include the # character at the beginning of the comment and do *not*
    /// include the line ending character.
    ///
    /// Comments may include non-UTF-8 characters. (This is why it's Vec<u8> and
    /// not String.)
    ///
    pub comments: Vec<(Vec<u8>, ByteIndex)>,

    ///
    /// Data about the markdown headers in the document.
    ///
    /// The key is the index of the BlockDelimiter token in the AST, and the value is
    /// the number of repeated = or - characters.
    ///
    pub inline_header_delimiters: HashMap<AstIndex, NonZeroU32>,
}

///
/// An index into [`Whitespace::whitespace_characters`]
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct WhitespaceIndex(NonZeroU32);

impl Default for CharData {
    fn default() -> Self {
        CharData {
            lines: DocumentLines::default(),
            whitespace_characters: StringInterner::new(),
            whitespace_ranges: Default::default(),
            comments: Default::default(),
            inline_header_delimiters: Default::default(),
        }
    }
}

// For StringInterner
impl Symbol for WhitespaceIndex {
    fn try_from_usize(val: usize) -> Option<Self> {
        if val < u32::MAX as usize {
            Some(WhitespaceIndex(unsafe {
                NonZeroU32::new_unchecked((val + 1) as u32)
            }))
        } else {
            None
        }
    }

    fn to_usize(self) -> usize {
        (self.0.get() as usize) - 1
    }
}

impl CharData {
    pub fn size(&self) -> Delta<ByteIndex> {
        Delta(self.lines.eof)
    }
}
