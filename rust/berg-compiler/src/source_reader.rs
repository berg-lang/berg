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
    fn open(&mut self, berg: &Berg) -> bool;
    fn index(&self) -> GraphemeIndex;
    fn peek(&mut self) -> Option<char>;
    fn read(&mut self) -> Option<char>;
    fn close(self) -> (SourceMetadata, CompileErrors);
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
    errors: CompileErrors,
    index: GraphemeIndex,
    peek_char: Option<char>,
    chars: Option<io::Chars<BufReader<File>>>,
}

pub struct StringSourceReader<'a> {
    source: &'a StringSource,
    metadata: SourceMetadata,
    errors: CompileErrors,
    index: GraphemeIndex,
    peek_char: Option<char>,
    chars: Option<str::Chars<'a>>,
}

impl<'a> FileSourceReader<'a> {
    pub fn new(source: &'a FileSource) -> FileSourceReader<'a> {
        let errors = CompileErrors::new();
        let metadata = SourceMetadata::new();
        FileSourceReader { source, metadata, errors, chars: None, index: 0, peek_char: None }
    }

    fn get_chars(&mut self) -> &mut io::Chars<BufReader<File>> {
        if let Some(ref mut chars) = self.chars {
            chars
        } else {
            panic!("Peek or read called on unopened FileSourceReader")
        }
    }

    fn open_file(&mut self, path: PathBuf) -> bool {
        match File::open(path) {
            Ok(file) => {
                self.chars = Some(BufReader::new(file).chars());
                self.peek_char = self.read_next_char();
                true
            },
            Err(error) => {
                let compile_error = match error.kind() {
                    ErrorKind::NotFound => SourceNotFound(error),
                    _ => IoOpenError(error)
                };
                self.report(compile_error);
                false
            },
        }
    }

    fn read_next_char(&mut self) -> Option<char> {
        match self.get_chars().next() {
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
    fn open(&mut self, berg: &Berg) -> bool {
        let path = self.source.path();
        if path.is_absolute() {
            self.open_file(path.clone())
        } else {
            match *berg.root() {
                Ok(ref root) => {
                    self.open_file(root.join(path))
                },
                Err(ref error) => {
                    let error_string = format!("{}", error);
                    self.report(IoCurrentDirectoryError(path.clone(), error.kind(), error_string));
                    false
                },
            }
        }
    }
    fn close(self) -> (SourceMetadata, CompileErrors) {
        (self.metadata, self.errors)
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
    pub fn new(source: &'a StringSource) -> StringSourceReader<'a> {
        let errors = CompileErrors::new();
        let metadata = SourceMetadata::new();
        StringSourceReader { source, metadata, errors, chars: None, index: 0, peek_char: None }
    }
    fn get_chars(&mut self) -> &mut str::Chars<'a> {
        if let Some(ref mut chars) = self.chars {
            chars
        } else {
            panic!("Peek or read called on unopened StringSourceReader")
        }
    }
    fn append_line(&mut self, index: GraphemeIndex) {
        self.metadata.append_line(index);
    }
}

impl<'a> SourceReader for StringSourceReader<'a> {
    fn open(&mut self, _: &Berg) -> bool {
        self.chars = Some(self.source.contents().chars());
        self.peek_char = self.get_chars().next();
        true
    }
    fn close(self) -> (SourceMetadata, CompileErrors) {
        (self.metadata, self.errors)
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
        self.peek_char = self.get_chars().next();
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
