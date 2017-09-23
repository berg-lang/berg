use parser::internals::*;

pub struct FileSourceReader<'a> {
    metadata: SourceMetadata<'a>,
    errors: CompileErrors,
    chars: Option<io::Chars<BufReader<File>>>,
    index: SourceIndex,
    peek_char: Option<char>,
}

impl<'a> FileSourceReader<'a> {
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
    fn append_line(&mut self, index: SourceIndex) {
        self.metadata.append_line(index);
    }
}

impl<'a> SourceReader<'a> for FileSourceReader<'a> {
    fn from_source(source: &'a Source) -> FileSourceReader<'a> {
        let errors = CompileErrors::new();
        let metadata = SourceMetadata::new(source);
        FileSourceReader { metadata, errors, chars: None, index: 0, peek_char: None }
    }
    fn open(&mut self, berg: &Berg) -> bool {
        if let Source::File(ref path) = *self.source() {
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
        } else {
            panic!("FileSourceReader passed a non-file!")
        }
    }
    fn close(self) -> (SourceMetadata<'a>, CompileErrors) {
        (self.metadata, self.errors)
    }
    fn report(&mut self, error: CompileError) {
        self.errors.report(error);
    }
    fn source(&self) -> &'a Source {
        self.metadata.source()
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
    fn index(&self) -> SourceIndex {
        self.index
    }
}
