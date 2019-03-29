use crate::parser::Tokenizer;
use crate::syntax::ExpressionBoundary::*;
use crate::syntax::OperatorToken::*;
use crate::syntax::ExpressionToken::*;
use crate::syntax::TermToken::*;
use crate::syntax::{Ast, ByteIndex, ByteSlice, ErrorTermError, IdentifierIndex, RawErrorTermError};
use crate::util::indexed_vec::Delta;
use std::str;
use ByteType::*;
use CharSeparation::*;
use CharType::*;
use InsignificantCharType::*;
use AnyCharType::*;

///
/// Iterates over character sequences in a buffer, returning each significant
/// token sequence as a [`Sequence`].
/// 
/// The whitespace, comments and indent between tokens is coalesced into a single
/// [`Separator`] value in [`Sequence.separator`].
/// 
/// The last whitespace in a file (if any) will not be yielded.
/// 
/// Also builds the char_data structure, storing whitespace sequences, comments and line endings.
/// 
///     use berg_compiler::parser::{*, SequenceType::*};
///     let mut sequencer = Sequencer::new("abc + b")
///     while let Some(sequence) = sequencer.next()) {
///         println!("Sequence {} preceded by {}")
///     }
///
#[derive(Debug)]
pub struct Sequencer<'p>(CharSequencer<'p>);

///
/// A significant (token-destined) sequence of bytes from the file.
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Sequence {
    ///
    /// The type of this sequence.
    /// 
    pub type: SequenceType,

    ///
    /// The whitespace before this sequence.
    /// 
    pub whitespace: PrecedingWhitespace,

    ///
    /// The starting position of this sequence (after the whitespace).
    /// 
    pub start: ByteIndex,
}

///
/// A significant character sequence.
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SequenceType {
    /// Identifier
    Identifier,
    /// Decimal digits (0-9)
    Integer,
    ///
    /// Series of operator characters (e.g. +, ++, >=).
    ///
    /// [`OpenParen`], [`CloseParen`], [`CloseCurly`], [`Separator`], and [`Colon`]
    /// are technically operator characters as well, but they have special rules
    /// so we class them differently.
    ///
    Operator,
    /// , or ;
    Separator,
    /// :
    Colon,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenCurly,
    /// }
    CloseCurly,
    ///
    /// Valid UTF-8 characters which aren't part of any recognized sequence and
    /// aren't whitespace.
    /// 
    /// Think emoji.
    /// 
    Unsupported,
    ///
    /// Invalid UTF-8 bytes.
    /// 
    /// Typically because the file is not actually UTF-8.
    /// 
    InvalidUtf8,
}

///
/// Characterizes the whitespace between tokens.
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
enum PrecedingWhitespace {
    ///
    /// Nothing between two tokens.
    /// 
    Empty,
    ///
    /// Horizontal space *only* between two tokens.
    ///
    Space,
    ///
    /// At least one newline between two tokens.
    /// 
    /// The parameter represents the indent level of the last line.
    /// 
    Indent(u32),
}

///
/// Chunks up the source into sequences: identifiers, numbers, space, newlines,
/// operators, etc.
/// 
/// In general, the sequencer chunks *character runs*--sequences containing all
/// of the same type of character. The most straightforward of these include:
///
/// | Integer | `1234` | A run of digit characters. | Term |
/// | Identifier | `ThisIsAnIdentifier` | A run of alphanumeric characters, or `_`. | Term |
/// | Operator | `+` `-` `*` `++` `+=` `<=>` `--->` | A run of operator characters. | Prefix if unbalanced like `+1`, postfix if unbalanced like If it's unbalanced like `+1` or `2*`, it's a postfix/prefix operator. Otherwise it's infix.
/// | Space | ` ` | A run of space characters. | Space |
/// | Whitespace | ` ` `\t` | A run of space and other whitespace characters. | Space |
/// | Unsupported | | A run of valid UTF-8 characters which aren't supported (such as emoji). | Term |
/// | Invalid | | A run of invalid UTF-8 bytes. | Term |
///
/// The sequencer also recognizes very specific, short character sequences, like
/// special operators and newlines, including:
///
/// | Separator | `;` `,` | Infix |
/// | Colon | `:` | Prefix if unbalanced like `:x`, otherwise infix.
/// | Open | `{` `(` | Open |
/// | Close | `}` `)` | Close |
/// | Newline | `\r` `\n` `\r\n` | Newlines are treated separately from other space, so that they can be counted for line #'s and possibly used to separate statements.
///
/// Responsible for building the char_data structure, storing whitespace
/// sequences, comments and line endings for debug information.
/// 
#[derive(Debug)]
struct CharSequencer<'p> {
    buffer: &'p [u8],
    next_char_end: ByteIndex,
    next_char_type: CharSequence,
    char_data: CharData,
}

///
/// A character sequence, including possible whitespace.
///
#[derive(Debug, Copy, Clone, PartialEq)]
enum CharSequence {
    ///
    /// Valid Sequences that will be returned by the calling Sequencer.
    ///
    Sequence(SequenceType),
    ///
    /// Horizontal space.
    /// 
    Space(SpaceChar),
    ///
    /// A line ending.
    /// 
    LineEnding(LineEndingChar),
    ///
    /// A comment (hash followed by all characters until the line ends)
    ///
    Comment,
}

///
/// A space character (space or tab).
/// 
enum SpaceChar {
    Space,
    Tab,
}

///
/// A line ending character (LF, CRLF, or CR).
/// 
#[derive(Debug, Copy, Clone, PartialEq)]
enum LineEndingChar {
    // LF (\n)
    Newline,
    // CR (\r)
    CarriageReturn,
    // CRLF (\r\n)
    CRLF,
}

impl<'p> ExpressionScanner<'p> {
    fn start(&mut self) -> 
}
impl<'p> Iterator for CharSequencer<'p> {
    type Item = CharSequence;

    fn next(&mut self) -> Option<CharSequence> {
        self.next_codepoint().map(|char_type| match char_type {

            Comment => self.scan_until(is_line_ending),
            LineEnding(CarriageReturn) => char_type,
        })
        map(|byte| match byte {
                Char(char_type) => char_type,
                CarriageReturn => {
                    let char_length = if let Some(&b'\n') = self.0.peek() {
                        2
                    } else {
                        1
                    };
                    (LineEnding, char_length.into())
                }
                Utf8LeadingByte(char_length) => {
                    if Self::is_valid_utf8_char(buffer, index, char_length) {
                        (Unsupported, char_length)
                    } else {
                        (InvalidUtf8, 1.into())
                    }
                }
            })
        // Depending on how the sequence starts, we'll do different things.
        match self.next_byte() {
            ByteType::
        }
    }
}

mod scanner {
    ///
    /// Classifies a UTF-8 source, byte by byte.
    /// 
    #[derive(Debug)]
    struct ByteScanner<'p> {
        /// The underlying buffer.
        next_index: ByteIndex,
    }

    ///
    /// Characters that never end up in tokens.
    /// 
    /// By "character" we mean *grapheme,* or a series of Unicode codepoints that
    /// form a single printable character.
    /// 
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum InsignificantCharType {
        ///
        /// #
        ///
        Hash,
        ///
        /// \n
        /// 
        /// See [`LineEnding`] for all other line ending characters.
        /// 
        Newline,
        ///
        /// \r and \r\n
        /// 
        /// See [`Newline`] for \n
        /// 
        LineEnding,
        ///
        /// " "
        /// 
        /// See [`HorizontalWhitespace`] for all other horizontal whitespace characters.
        /// 
        Space,
        ///
        /// \t
        /// 
        /// See [`Space`] for the single space whitespace character.
        /// 
        HorizontalWhitespace,
    }

    ///
    /// Type of a byte.
    /// 
    #[derive(Debug, Copy, Clone, PartialEq)]
    enum ByteType {
        ///
        /// A byte that is likely a part of the given character sequence
        /// 
        Char(CharSequence),
        ///
        /// The first byte of a multibyte non-ASCII UTF-8 codepoint.
        /// 
        /// The parameter represents how many bytes the codepoint is.
        /// 
        Utf8LeadingByte(Delta<ByteIndex>),
    }

    ///
    /// The next significant character.
    /// 
    struct NextChar {
        ///
        /// The separation between the previous character and this one.
        /// 
        separation: CharSeparation,
        ///
        /// The character type.
        ///
        char_type: CharType,
        ///
        /// The starting position of the character.
        /// 
        /// The ending position can be found at [`Scanner::index`].
        /// 
        start: ByteIndex,
    }

    ///
    /// Characterizes the separation between significant characters.
    ///
    #[derive(Debug, Clone)]
    pub enum SequenceSeparator {
        ///
        /// There is no space before the next significant character.
        ///
        NoSeparation,
        ///
        /// There is space before the next significant character, but it is on the
        /// same line.
        ///
        /// The parameter represents the amount of space.
        /// 
        SameLine,
        ///
        /// The next significant character is on another line.
        ///
        /// The parameter represents the indent level of the next significant
        /// character. May be zero.
        /// 
        DifferentLine(Delta<ByteIndex>),
        ///
        /// The next significant character is on another line, and there are comment
        /// lines in that are *less* indented and thus might cause undent.
        /// 
        /// The first parameter represents the indent level of the next significant
        /// character. Will NOT be zero.
        /// 
        /// The second parameter represents the lowest comment undent level. Will be
        /// smaller than the first parameter.
        /// 
        DifferentLineWithUndent(Delta<ByteIndex>, Delta<ByteIndex>),
    }

    impl<Input: Iterator<Item=ByteType>> Iterator for CharScanner<Input> {
        type Item = CharType;
        fn next(&mut self) -> Option<Self::Item> {
            self.0.next().map(|byte| match byte {
                Char(char_type) => char_type,
                CarriageReturn => {
                    let char_length = if let Some(&b'\n') = self.0.peek() {
                        2
                    } else {
                        1
                    };
                    (LineEnding, char_length.into())
                }
                Utf8LeadingByte(char_length) => {
                    if Self::is_valid_utf8_char(buffer, index, char_length) {
                        (Unsupported, char_length)
                    } else {
                        (InvalidUtf8, 1.into())
                    }
                }
            })
        }
    }

    impl<Input: Iterator<Item=(ByteIndex,CharType)>> Iterator for BasicSequencer<Input> {
        type Item = (ByteIndex, BasicSequence);
        fn next(&mut self) -> Option<Self::Item> {

        }
    }

    impl<Input: Iterator<Item=(ByteIndex,CharType)>> Iterator for Sequencer<Input> {
        type Item = (SequenceSeparator, ByteIndex, Sequence);
    }
    impl<'a> Sequencer<'a> {
        pub fn new(ast: Ast<'a>) -> Self {
            Sequencer {
                tokenizer: Tokenizer::new(ast),
            }
        }

        pub fn parse_buffer(mut self, buffer: &ByteSlice) -> Ast<'a> {
            // Record the buffer size.
            self.ast_mut().char_data.size = buffer.len().into();

            // Read whitespace at the beginning of the file and send indent info
            let mut scanner = CharScanner::new(buffer);
            assert!(scanner.index == 0);
            let next = self.next_char(buffer, &mut scanner);
            let indent = match next {
                Some(NextChar { separation: NoSeparation, .. }) => Some(0.into()),
                Some(NextChar { separation: SameLine, start, .. }) => Some(Delta(start)),
                Some(NextChar { separation: DifferentLine(indent), .. }) => Some(indent),
                Some(NextChar { separation: DifferentLineWithUndent(..), .. }) => unreachable!(),
                None => None,
            };
            self.tokenizer.on_source_start(indent, scanner.index);

            let bytes = buffer.iter().enumerate().map(|(i,b)| (i,ByteType::from_byte(b)));
            let chars = bytes.try_fold();

            // Read all the tokens
            while let Some(next_char) = next {
                next = self.next_sequence(next_char, buffer, &mut scanner);
            }

            // Send the "source end" token
            assert!(scanner.index == buffer.len());
            self.tokenizer.on_source_end(scanner.index)
        }

        pub fn ast_mut(&mut self) -> &mut Ast<'a> {
            self.tokenizer.ast_mut()
        }

        fn next_char(&mut self, buffer: &ByteSlice, scanner: &mut Scanner) -> Option<NextChar> {
            // First, see if the next character is significant.
            let start = scanner.index;
            let space = match scanner.read() {
                Some(Significant(char_type)) => return Some(NextChar { separation: NoSeparation, char_type, start }),
                None => return None,
                Some(Insignificant(char_type)) => char_type,
            };

            // Next, we skip any horizontal-only whitespace
            let mut line_ending = match scanner.skip_horizontal_space(space, buffer, scanner) {
                Some((Insignificant(char_type), start)) => (char_type, start),
                Some((Significant(char_type), start)) => return Some(NextChar { separation: SameLine, char_type, start }),
                None => return None,
            };

            // Finally, we loop through all lines and calculate indent
            loop {
                let line_start = scanner.index;
                line_ending = match scanner.skip_horizontal_space(space, buffer, scanner) {
                    Some((Insignificant(char_type), start)) => (char_type, start),
                    Some((Significant(char_type), start)) => return Some(NextChar { separation: DifferentLine(start - line_start), char_type, start }),
                    None => return None,
                };
            }
        }

        fn read_whitespace(&mut self, first_char: NextChar<InsignificantCharType>, buffer: &ByteSlice, scanner: &mut Scanner) -> Option<(CharSeparation, NextChar)> {
            if !first_char.char_type.ends_line() {
                match self.read_until_eol(buffer, scanner) {
                    Some(NextChar { char_type: Insignificant(char_type), start })
                }
            }
            match self.read_until_eol() {

            }
            let whitespace = next_char;
            while !whitespace.char_type.ends_line() {
                match scanner.read(buffer) {
                    Some(NextChar { char_type: Insignificant(char_type), start }) => { whitespace = NextChar { char_type, start }; },
                    Some(NextChar { char_type: Significant(char_type), start }) => return Some((CharSeparation, NextChar { char_type, start }))
                }
            }
            while !whitespace.char_type
        }

        fn next_token(&mut self, buffer: &ByteSlice, scanner: &mut Scanner) -> bool {
            let start = scanner.index;
            if let Some(char_type, whitespace) = scanner.next(buffer) {
                match char_type {
                    Digit => self.integer(buffer, start, scanner),
                    Identifier => self.identifier(buffer, start, scanner),
                    Operator => self.operator(buffer, start, scanner),
                    Separator => self.separator(buffer, start, scanner),
                    Colon => self.colon(buffer, start, scanner),
                    OpenParen => self.tokenizer.on_open(Parentheses, start..scanner.index),
                    CloseParen => self.tokenizer.on_close(Parentheses, start..scanner.index),
                    OpenCurly => self.tokenizer.on_open(CurlyBraces, start..scanner.index),
                    CloseCurly => self.tokenizer.on_close(CurlyBraces, start..scanner.index),
                    Unsupported => self.unsupported(buffer, start, scanner),
                    InvalidUtf8 => self.invalid_utf8(buffer, start, scanner),
                };
                true
            } else {
                false
            }
        }

        fn utf8_syntax_error(
            &mut self,
            error: ErrorTermError,
            buffer: &ByteSlice,
            start: ByteIndex,
            scanner: &Scanner,
        ) {
            let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
            let literal = self.ast_mut().literals.get_or_intern(string);
            self.tokenizer
                .on_term_token(ErrorTerm(error, literal), start..scanner.index);
        }

        fn raw_syntax_error(
            &mut self,
            error: RawErrorTermError,
            buffer: &ByteSlice,
            start: ByteIndex,
            scanner: &Scanner,
        ) {
            let raw_literal = self
                .ast_mut()
                .raw_literals
                .push(buffer[start..scanner.index].into());
            self.tokenizer
                .on_term_token(RawErrorTerm(error, raw_literal), start..scanner.index);
        }

        fn integer(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while(Digit, buffer);
            if scanner.next_while_identifier(buffer) {
                return self.utf8_syntax_error(
                    ErrorTermError::IdentifierStartsWithNumber,
                    buffer,
                    start,
                    scanner,
                );
            }
            let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
            let literal = self.ast_mut().literals.get_or_intern(string);
            self.tokenizer
                .on_term_token(IntegerLiteral(literal), start..scanner.index)
        }

        fn identifier(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while_identifier(buffer);
            let string = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
            let identifier = self.ast_mut().intern_identifier(string);
            self.tokenizer
                .on_term_token(RawIdentifier(identifier), start..scanner.index)
        }

        fn make_identifier(&mut self, slice: &[u8]) -> IdentifierIndex {
            let string = unsafe { str::from_utf8_unchecked(slice) };
            self.ast_mut().intern_identifier(string)
        }

        fn operator(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while(Operator, buffer);

            // If we're at the start of a term, this operator is prefix. e.g. ("-1", "1 + -1", "(-1)", "1,-2", "a:-2")
            // If we're at the end of a term, this operator is postfix. e.g. ("a?", "a? + 2", "a,? , 2")
            // If we're in the middle of a term, this operator is infix. e.g. ("1-2", "a-:b")
            // If we're not part of a term, this operator is infix. e.g. ("1 + 2")
            let term_is_about_to_end = {
                let char_type = scanner.peek(buffer);
                !char_type.can_be_right_operand()
                    || char_type.is_close()
                    || char_type == Separator
                    || (char_type == Colon && scanner.peek_at(buffer, 1).can_be_right_operand())
            };

            if self.tokenizer.in_term && term_is_about_to_end {
                let identifier = self.make_identifier(&buffer[start..scanner.index]);
                self.tokenizer
                    .on_term_token(PostfixOperator(identifier), start..scanner.index);
            // If we're *not* in a term, and there is something else right after the
            // operator, it is prefix. i.e. "+1"
            } else if !self.tokenizer.in_term && !term_is_about_to_end {
                let identifier = self.make_identifier(&buffer[start..scanner.index]);
                self.tokenizer
                    .on_term_token(PrefixOperator(identifier), start..scanner.index);
            // Otherwise, it's infix. i.e. "1+2" or "1 + 2"
            } else {
                let token = if Self::is_assignment_operator(&buffer[start..scanner.index]) {
                    InfixAssignment(self.make_identifier(&buffer[start..scanner.index - 1]))
                } else {
                    InfixOperator(self.make_identifier(&buffer[start..scanner.index]))
                };
                // If the infix operator is like a+b, it's inside the term. If it's
                // like a + b, it's outside (like a separator).
                if self.tokenizer.in_term {
                    self.tokenizer.on_term_token(token, start..scanner.index);
                } else {
                    self.tokenizer.on_separator(token, start..scanner.index);
                }
            }
        }

        fn separator(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            let string = self.make_identifier(&buffer[start..scanner.index]);
            self.tokenizer
                .on_separator(InfixOperator(string), start..scanner.index)
        }

        // Colon is, sadly, just a little ... special.
        // If the colon is an operand or is at the start of the term ("1 + :a, 1+:a, a :b"), it's prefix.
        // Otherwise, it's a separator.
        // See where the "operator" function calculates whether the term is about to end for the other
        // relevant silliness to ensure "a+:b" means "(a) + (:b)".
        fn colon(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            let identifier = self.make_identifier(&buffer[start..scanner.index]);
            if (!self.tokenizer.in_term || self.tokenizer.prev_was_operator)
                && scanner.peek(buffer).is_always_right_operand()
            {
                self.tokenizer
                    .on_term_token(PrefixOperator(identifier), start..scanner.index);
            } else {
                self.tokenizer
                    .on_separator(InfixOperator(identifier), start..scanner.index);
            }
        }

        // Anything ending with exactly one = is assignment, EXCEPT
        // >=, != and <=.
        fn is_assignment_operator(slice: &[u8]) -> bool {
            if slice[slice.len() - 1] != b'=' {
                return false;
            }
            if slice.len() < 2 {
                return true;
            }
            let prev_ch = slice[slice.len() - 2];
            if prev_ch == b'=' {
                return false;
            }
            if slice.len() > 2 {
                return true;
            }
            match prev_ch {
                b'!' | b'>' | b'<' => false,
                _ => true,
            }
        }

        fn insignificant(&mut self, char_type: CharType, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            assert!(char_type)
                Hash => self.comment(buffer, start, scanner),
                Newline => self.newline(buffer, start, scanner),
                LineEnding => self.line_ending(buffer, start, scanner),
                Space => self.space(buffer, start, scanner),
                HorizontalWhitespace => self.horizontal_whitespace(buffer, start, scanner),
                Eof => return false,
        }

        fn newline(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            self.tokenizer.on_line_ending(start..scanner.index);
            self.line_start(buffer, scanner)
        }

        fn line_ending(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            self.store_whitespace(buffer, start, scanner);
            self.tokenizer.on_line_ending(start..scanner.index);
            self.line_start(buffer, scanner)
        }

        ///
        /// Does several things every time a line starts:
        /// 
        /// 1. Adds the line start position to the AST so line positions can be calculated.
        /// 2. Processes horizontal whitespace (indent).
        /// 3. If the line is non-empty, sends on_indent.
        /// 
        fn line_start(&mut self, buffer: &ByteSlice, scanner: &mut Scanner) {
            // Push the line start
            self.ast_mut().char_data.line_starts.push(scanner.index);

            // Check for indent
            let start = scanner.index;
            if scanner.next_if(Space, buffer) {
                self.space(buffer, start, scanner);
            } else if scanner.next_if(HorizontalWhitespace, buffer) {
                self.horizontal_whitespace(buffer, start, scanner);
            }

            // Send "indent" unless the line is just whitespace
            if !scanner.peek(buffer).ends_line() {
                self.tokenizer.on_indent(start..scanner.index)
            }
        }

        fn space(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while(Space, buffer);
            // Only store spaces in char data if it's mixed space
            if scanner.next_while_horizontal_whitespace(buffer) {
                self.store_whitespace(buffer, start, scanner);
            }
            self.tokenizer.on_space(start)
        }

        fn horizontal_whitespace(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while_horizontal_whitespace(buffer);
            self.store_whitespace(buffer, start, scanner);
            self.tokenizer.on_space(start)
        }

        // # <comment>
        fn comment(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_until_eol(buffer);
            self.ast_mut().char_data.append_comment(
                &buffer[start..scanner.index],
                start,
            );
            self.tokenizer.on_space(start)
        }

        fn unsupported(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while(Unsupported, buffer);
            self.utf8_syntax_error(ErrorTermError::UnsupportedCharacters, buffer, start, scanner)
        }

        fn invalid_utf8(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &mut Scanner) {
            scanner.next_while(InvalidUtf8, buffer);
            self.raw_syntax_error(RawErrorTermError::InvalidUtf8, buffer, start, scanner)
        }

        fn store_whitespace(&mut self, buffer: &ByteSlice, start: ByteIndex, scanner: &Scanner) {
            let whitespace = unsafe { str::from_utf8_unchecked(&buffer[start..scanner.index]) };
            self.ast_mut().char_data.append_whitespace(whitespace, start)
        }
    }

    impl<'p> Iterator for CharScanner<'p> {
        type Item = CharType;
        fn new(&mut self, buffer: &ByteSlice) -> CharType {
            CharScanner { byte_scanner: ByteScanner}
        }
        fn next(&mut self, buffer: &ByteSlice) -> CharType {
            let (char_type, char_length) = CharType::read(buffer, self.index);
            if char_length == 0 {
                assert!(char_type == Eof);
            } else {
                self.advance(char_length);
            }
            char_type
        }
    }

    impl<'p> CharScanner<'p> {
        fn peek(&self, buffer: &ByteSlice) -> CharType {
            CharType::peek(buffer, self.index)
        }

        fn peek_at<At: Into<Delta<ByteIndex>>>(&self, buffer: &ByteSlice, delta: At) -> CharType {
            CharType::peek(buffer, self.index + delta.into())
        }

        fn next_while(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
            if self.next_if(if_type, buffer) {
                while self.next_if(if_type, buffer) {}
                true
            } else {
                false
            }
        }

        fn next_until_eol(&mut self, buffer: &ByteSlice) {
            loop {
                let (char_type, char_length) = CharType::read(buffer, self.index);
                if char_type.ends_line() {
                    return;
                }
                self.advance(char_length);
            }
        }

        fn next_while_horizontal_whitespace(&mut self, buffer: &ByteSlice) -> bool {
            let mut found = false;
            loop {
                let (char_type, char_length) = CharType::read(buffer, self.index);
                if char_type.is_horizontal_whitespace() {
                    self.advance(char_length);
                    found = true;
                } else {
                    break;
                }
            }
            found
        }

        fn next_while_identifier(&mut self, buffer: &ByteSlice) -> bool {
            let mut found = false;
            loop {
                let (char_type, char_length) = CharType::read(buffer, self.index);
                if char_type.is_identifier_middle() {
                    self.advance(char_length);
                    found = true;
                } else {
                    break;
                }
            }
            found
        }

        fn next_if(&mut self, if_type: CharType, buffer: &ByteSlice) -> bool {
            let (char_type, char_length) = CharType::read(buffer, self.index);
            if char_type == if_type {
                self.advance(char_length);
                true
            } else {
                false
            }
        }

        fn advance(&mut self, char_length: Delta<ByteIndex>) {
            assert!(char_length > 0);
            self.index += char_length;
        }
    }

    impl CharType {
        fn read(buffer: &ByteSlice, index: ByteIndex) -> (CharType, Delta<ByteIndex>) {
            if let Some(byte_type) = ByteType::peek(buffer, index) {
                match byte_type {
                    Char(char_type) => (char_type, 1.into()),
                    CarriageReturn => {
                        let char_length = if let Some(&b'\n') = buffer.get(index + 1) {
                            2
                        } else {
                            1
                        };
                        (LineEnding, char_length.into())
                    }
                    ByteType::Utf8LeadingByte(char_length) => {
                        if Self::is_valid_utf8_char(buffer, index, char_length) {
                            (Unsupported, char_length)
                        } else {
                            (InvalidUtf8, 1.into())
                        }
                    }
                }
            } else {
                (Eof, 0.into())
            }
        }

        fn peek(buffer: &ByteSlice, index: ByteIndex) -> CharType {
            CharType::read(buffer, index).0
        }

        fn is_valid_utf8_char(
            buffer: &ByteSlice,
            index: ByteIndex,
            char_length: Delta<ByteIndex>,
        ) -> bool {
            if index + char_length > buffer.len() {
                return false;
            }
            match char_length {
                Delta(ByteIndex(2)) => ByteType::is_utf8_cont(buffer[index + 1]),
                Delta(ByteIndex(3)) => {
                    ByteType::is_utf8_cont(buffer[index + 1])
                        && ByteType::is_utf8_cont(buffer[index + 2])
                }
                Delta(ByteIndex(4)) => {
                    ByteType::is_utf8_cont(buffer[index + 1])
                        && ByteType::is_utf8_cont(buffer[index + 2])
                        && ByteType::is_utf8_cont(buffer[index + 3])
                }
                _ => unreachable!(),
            }
        }

        pub(crate) fn is_identifier_middle(self) -> bool {
            match self {
                Identifier | Digit => true,
                _ => false,
            }
        }

        pub(crate) fn is_significant(self) -> bool {
            match self {
                Identifier | Digit | Operator | Separator | OpenCurly | OpenParen |
                CloseCurly | CloseParen | Colon | InvalidUtf8 | Unsupported => true,
                Space | HorizontalWhitespace | Newline | Eof => false,
            }
        }

        pub(crate) fn is_horizontal_whitespace(self) -> bool {
            match self {
                Space | HorizontalWhitespace => true,
                Identifier | Digit | Operator | Separator | OpenCurly | OpenParen |
                CloseCurly | CloseParen | Colon | InvalidUtf8 | Unsupported |
                Newline | LineEnding | Eof => false,
            }
        }

        pub(crate) fn ends_line(self) -> bool {
            match self {
                Newline | LineEnding | Eof => true,
                Identifier | Digit | InvalidUtf8 | Unsupported |
                Operator | Separator | Colon |
                OpenCurly | OpenParen |
                CloseCurly | CloseParen |
                Space | HorizontalWhitespace => false,
            }
        }

        pub(crate) fn is_always_postfix(self) -> bool {
            match self {
                CloseParen | CloseCurly => true,
                Identifier | Digit | InvalidUtf8 | Unsupported |
                Operator | Separator | Colon |
                OpenCurly | OpenParen |
                Space | HorizontalWhitespace | Newline | LineEnding | Eof => false,
            }
        }

        pub(crate) fn is_always_prefix(self) -> bool {
            match self {
                OpenParen | OpenCurly => true,
                Digit | Identifier | InvalidUtf8 | Unsupported |
                Operator | Separator | OpenCurly | OpenParen |
                CloseCurly | CloseParen | Colon |
                Space | HorizontalWhitespace | Newline | LineEnding | Eof => false,
            }
        }

        pub(crate) fn is_always_term(self) -> bool {
            match self {
                Digit | Identifier | InvalidUtf8 | Unsupported => true,
                Operator | Separator | OpenCurly | OpenParen |
                CloseCurly | CloseParen | Colon |
                Space | HorizontalWhitespace | Newline | LineEnding | Eof => false,
            }
        }

        pub(crate) fn can_have_left_operand(self) -> bool {
            match self {
                Operator | Separator | Colon |
                CloseCurly | CloseParen => true,
                Digit | Identifier | InvalidUtf8 | Unsupported |
                OpenCurly | OpenParen |
                Space | HorizontalWhitespace | Newline | LineEnding | Eof => false,
            }
        }

        pub(crate) fn ends_term(self) -> bool {
            match self {
                Digit | Identifier | InvalidUtf8 | Unsupported => true,
                Operator | Separator | OpenCurly | OpenParen |
                CloseCurly | CloseParen | Colon |
                Space | HorizontalWhitespace | Newline | LineEnding | Eof => false,
            }
        }
    }

    impl<'p> Iterator for ByteScanner<'p> {
        type Item = ByteType;
        fn next(&mut self) -> Option<ByteType> {
            self.buffer.get(self.index).map(|b| ByteType::from_byte(*b))
        }
    }

    impl<'p> ByteScanner<'p> {
        fn new(buffer: &'p ByteSlice) -> Self {
            ByteScanner { buffer, index: 0 }
        }
        fn buffer(&self) -> &ByteSlice {
            self.buffer
        }
    }

    impl ByteType {
        fn from_byte(byte: u8) -> ByteType {
            match byte {
                b'+' | b'-' | b'*' | b'/' | b'=' | b'>' | b'<' | b'&' | b'|' | b'!' | b'.' => {
                    Char(Operator)
                }
                b'0'...b'9' => Char(Digit),
                b'a'...b'z' | b'A'...b'Z' | b'_' => Char(Identifier),
                b'(' => Char(OpenParen),
                b'{' => Char(OpenCurly),
                b')' => Char(CloseParen),
                b'}' => Char(CloseCurly),
                b';' | b',' => Char(Separator),
                b':' => Char(Colon),
                b'#' => Char(Hash),
                b' ' => Char(Space),
                b'\t' => Char(HorizontalWhitespace),
                b'\n' => Char(Newline),
                b'\r' => ByteType::CarriageReturn,
                _ => ByteType::generic(byte),
            }
        }

        fn generic(byte: u8) -> Self {
            match byte {
                0b0000_0000...0b0111_1111 => Char(CharType::Unsupported),
                0b1100_0000...0b1101_1111 => Utf8LeadingByte(Delta(ByteIndex(2))),
                0b1110_0000...0b1110_1111 => Utf8LeadingByte(Delta(ByteIndex(3))),
                0b1111_0000...0b1111_0111 => Utf8LeadingByte(Delta(ByteIndex(4))),
                _ => Char(CharType::InvalidUtf8),
            }
        }

        fn is_utf8_cont(byte: u8) -> bool {
            byte >= 0b1000_0000 && byte < 0b1011_1111
        }
    }
}