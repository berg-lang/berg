use berg::*;
use compile_errors::*;
use std::fs::File;
use std::io;
use std::io::{BufReader,CharsError,ErrorKind,Read};
use std::ops::Range;
use std::path::PathBuf;
use std::str;

/// Stream wrapper built for parsing:
/// - reports errors when UTF-8 is bad or I/O fails
/// - sets line starts information for row/column printing information
/// - sets file size
/// - ensures stream is peekable

pub trait SourceReader {
    fn index(&self) -> GraphemeIndex;
    fn peek(&mut self) -> Option<char>;
    fn read(&mut self) -> Option<char>;
    fn close(self) -> SourceMetadata;
    fn report(&mut self, error: CompileError);
    fn source<'a>(&'a self) -> &'a Source;

    fn read_if(&mut self, valid_char: fn(char) -> bool) -> Option<char> {
        if let Some(ch) = self.peek() {
            if valid_char(ch) {
                return self.read();
            }
        }
        None
    }
    fn read_while(&mut self, valid_char: fn(char) -> bool, string: &mut String) -> bool {
        if let Some(ch) = self.read_if(valid_char) {
            string.push(ch);
            while let Some(ch) = self.read_if(valid_char) {
                string.push(ch);
            }
            true
        } else {
            false
        }
    }
}

pub struct FileSourceReader<'a> {
    source: &'a FileSource,
    metadata: SourceMetadata,
    errors: &'a mut CompileErrors,
    index: GraphemeIndex,
    peek_char: Option<char>,
    chars: io::Chars<BufReader<File>>,
}

pub struct StringSourceReader<'a> {
    source: &'a StringSource,
    metadata: SourceMetadata,
    errors: &'a mut CompileErrors,
    index: GraphemeIndex,
    peek_char: Option<char>,
    chars: str::Chars<'a>,
}

impl<'a> FileSourceReader<'a> {
    pub fn open(source: &'a FileSource, errors: &'a mut CompileErrors, berg: &Berg) -> Option<FileSourceReader<'a>> {
        let path = source.path();
        if path.is_absolute() {
            Self::open_file(path.clone(), source, errors)
        } else {
            match *berg.root() {
                Ok(ref root) => {
                    Self::open_file(root.join(path), source, errors)
                },
                Err(ref error) => {
                    let error_string = format!("{}", error);
                    errors.report(IoCurrentDirectoryError(path.clone(), error.kind(), error_string));
                    None
                },
            }
        }
    }

    fn open_file(path: PathBuf, source: &'a FileSource, errors: &'a mut CompileErrors) -> Option<FileSourceReader<'a>> {
        match File::open(path) {
            Ok(file) => {
                let metadata = SourceMetadata::new();
                let chars = BufReader::new(file).chars();
                let mut reader = FileSourceReader { source, metadata, errors, chars, index: 0, peek_char: None };
                reader.peek_char = reader.read_next_char();
                Some(reader)
            },
            Err(error) => {
                let compile_error = match error.kind() {
                    ErrorKind::NotFound => SourceNotFound(error),
                    _ => IoOpenError(error)
                };
                errors.report(compile_error);
                None
            },
        }
    }

    fn read_next_char(&mut self) -> Option<char> {
        match self.chars.next() {
            Some(Ok(ch)) => {
                self.index += 1;
                Some(ch)
            }
            None => None,
            Some(Err(CharsError::NotUtf8)) => {
                let error = InvalidUtf8(Range { start: self.index, end: self.index });
                self.report(error);
                self.index += 1;
                None
            },
            Some(Err(CharsError::Other(error))) => {
                let error = IoReadError(self.index, error);
                self.report(error);
                None
            },
        }
    }
    fn append_line(&mut self, index: GraphemeIndex) {
        self.metadata.append_line(index);
    }
}

impl<'a> SourceReader for FileSourceReader<'a> {
    fn close(self) -> SourceMetadata {
        self.metadata
    }
    fn report(&mut self, error: CompileError) {
        self.errors.report(error);
    }
    fn source<'b>(&'b self) -> &'b Source {
        self.source
    }

    fn peek(&mut self) -> Option<char> {
        self.peek_char
    }
    fn read(&mut self) -> Option<char> {
        let result = self.peek_char;
        self.peek_char = self.read_next_char();
        if self.peek_char.is_some() {
            self.index += 1;
        }
        let index = self.index;
        match (result, self.peek_char) {
            (Some('\r'), Some('\n')) => self.append_line(index + 1),
            (Some('\r'), Some(_)) => self.append_line(index),
            (Some('\n'), Some(_)) => self.append_line(index),

            (Some('\r'), None) => self.append_line(index + 1),
            (Some('\n'), None) => self.append_line(index + 1),

            _ => {},
        }
        result
    }
    fn index(&self) -> GraphemeIndex {
        self.index
    }
}

impl<'a> StringSourceReader<'a> {
    pub fn open(source: &'a StringSource, errors: &'a mut CompileErrors) -> StringSourceReader<'a> {
        let metadata = SourceMetadata::new();
        let mut chars = source.contents().chars();
        let peek_char = chars.next();
        StringSourceReader { source, metadata, errors, chars, index: 0, peek_char }
    }
    fn append_line(&mut self, index: GraphemeIndex) {
        self.metadata.append_line(index);
    }
}

impl<'a> SourceReader for StringSourceReader<'a> {
    fn close(self) -> SourceMetadata {
        self.metadata
    }
    fn report(&mut self, error: CompileError) {
        self.errors.report(error);
    }
    fn source<'b>(&'b self) -> &'b Source {
        self.source
    }

    fn peek(&mut self) -> Option<char> {
        self.peek_char
    }
    fn read(&mut self) -> Option<char> {
        // TODO it is gross that we repeat all this logic.
        let result = self.peek_char;
        self.peek_char = self.chars.next();
        if self.peek_char.is_some() {
            self.index += 1;
        }
        let index = self.index;
        match (result, self.peek_char) {
            (Some('\r'), Some('\n')) => self.append_line(index + 1),
            (Some('\r'), Some(_)) => self.append_line(index),
            (Some('\n'), Some(_)) => self.append_line(index),

            (Some('\r'), None) => self.append_line(index + 1),
            (Some('\n'), None) => self.append_line(index + 1),

            _ => {},
        }
        result
    }
    fn index(&self) -> GraphemeIndex {
        self.index
    }
}
