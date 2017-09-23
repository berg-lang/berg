use parser::internals::*;

/// Stream wrapper built for parsing:
/// - reports errors when UTF-8 is bad or I/O fails
/// - sets line starts information for row/column printing information
/// - sets file size
/// - ensures stream is peekable

pub trait SourceReader<'a> {
    fn from_source(source: &'a Source) -> Self;
    fn open(&mut self, berg: &Berg) -> bool;
    fn index(&self) -> SourceIndex;
    fn peek(&mut self) -> Option<char>;
    fn read(&mut self) -> Option<char>;
    fn close(self) -> (SourceMetadata<'a>, CompileErrors);
    fn report(&mut self, error: CompileError);
    fn source(&self) -> &'a Source;

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
