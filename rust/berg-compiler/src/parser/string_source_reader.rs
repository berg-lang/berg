use parser::internals::*;

pub struct StringSourceReader<'a> {
    metadata: SourceMetadata<'a>,
    errors: CompileErrors,
    chars: Option<str::Chars<'a>>,
    index: SourceIndex,
    peek_char: Option<char>,
}

impl<'a> StringSourceReader<'a> {
    fn get_chars(&mut self) -> &mut str::Chars<'a> {
        if let Some(ref mut chars) = self.chars {
            chars
        } else {
            panic!("Peek or read called on unopened StringSourceReader")
        }
    }
    fn append_line(&mut self, index: SourceIndex) {
        self.metadata.append_line(index);
    }
}

impl<'a> SourceReader<'a> for StringSourceReader<'a> {
    fn from_source(source: &'a Source) -> StringSourceReader<'a> {
        let errors = CompileErrors::new();
        let metadata = SourceMetadata::new(source);
        StringSourceReader { metadata, errors, chars: None, index: 0, peek_char: None }
    }
    fn open(&mut self, _: &Berg) -> bool {
        if let Source::String(_, ref contents) = *self.source() {
            self.chars = Some(contents.chars());
            self.peek_char = self.get_chars().next();
            true
        } else {
            panic!("StringSourceReader passed a non-string!")
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
    fn index(&self) -> SourceIndex {
        self.index
    }
}
