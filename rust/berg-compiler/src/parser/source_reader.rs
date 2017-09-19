use parser::internals::*;

/// Stream wrapper built for parsing:
/// - reports errors when UTF-8 is bad or I/O fails
/// - sets line starts information for row/column printing information
/// - sets file size
/// - ensures stream is peekable
pub struct SourceReader {
    pub metadata: SourceMetadata,
    pub errors: CompileErrors,
    pub index: usize,
    chars: SourceReaderChars,
    peek_char: Option<char>,
}

enum SourceReaderChars {
    Unreadable,
    File(Chars<BufReader<File>>),
}

impl SourceReader {
    pub fn start(source: &Source, mut errors: CompileErrors) -> SourceReader {
        let metadata = SourceMetadata::new(0);
        let index = 0;
        let mut chars = SourceReaderChars::open(source, &mut errors);

        // Read the first character so peek works
        let peek_char = chars.next(index, &mut errors);

        SourceReader { chars, metadata, index, peek_char, errors }
    }
    pub fn close(self) -> (SourceMetadata, CompileErrors) {
        (self.metadata, self.errors)
    }
    pub fn peek(&mut self) -> Option<char> {
        self.peek_char
    }
    pub fn read(&mut self) -> Option<char> {
        if let Some(prev_char) = self.peek_char {
            self.index += 1;
            self.peek_char = self.chars.next(self.index, &mut self.errors);
            Some(prev_char)
        } else {
            None
        }
    }
    pub fn read_if(&mut self, valid_char: fn(char) -> bool) -> Option<char> {
        if let Some(ch) = self.peek_char {
            if valid_char(ch) {
                return self.read();
            }
        }
        None
    }
    pub fn read_while(&mut self, valid_char: fn(char) -> bool, string: &mut String) -> bool {
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
    pub fn report(&mut self, error: CompileError) {
        self.errors.report(error);
    }
}

impl SourceReaderChars {
    fn open(source: &Source, errors: &mut CompileErrors) -> SourceReaderChars {
        match source {
            &Source::File(ref path) => {
                match File::open(&path) {
                    Ok(file) => {
                        let chars = BufReader::new(file).chars();
                        SourceReaderChars::File(chars)
                    },
                    Err(error) => {
                        let compile_error = match error.kind() {
                            ErrorKind::NotFound => SourceNotFound(error),
                            _ => IoOpenError(error)
                        };
                        errors.report(compile_error);
                        SourceReaderChars::Unreadable
                    },
                }
            },
        }
    }
    // Really really want this inline so we aren't passing these parameters all the time.
    #[inline(always)]
    fn next(&mut self, index: usize, errors: &mut CompileErrors) -> Option<char> {
        match self {
            &mut SourceReaderChars::File(ref mut chars) => {
                match chars.next() {
                    Some(Ok(ch)) => Some(ch),
                    Some(Err(CharsError::NotUtf8)) => { errors.report(InvalidUtf8(Range { start: index, end: index })); None }
                    Some(Err(CharsError::Other(error))) => { errors.report(IoReadError(index, error)); None }
                    None => None,
                }
            },
            &mut SourceReaderChars::Unreadable => None
        }
    }
}