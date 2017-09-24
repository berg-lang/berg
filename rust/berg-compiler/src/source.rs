use berg::*;
use compile_errors::*;
use parser::Parser;
use source_reader::*;
use std::path::PathBuf;
use std::ffi::OsStr;

pub trait Source {
    fn name<'a>(&'a self) -> &'a OsStr;
    // TODO we need (or want) this so that the parser gets constructed with the
    // specific source-type's implementation, but this is icky and we didn't want it
    // exposed to the public in the first place.
    fn parse(&self, berg: &Berg) -> ParseResult;
}

#[derive(Debug)]
pub struct FileSource {
    path: PathBuf,
}

#[derive(Debug)]
pub struct StringSource {
    name: String,
    contents: String,
}

impl FileSource {
    pub fn new(path: PathBuf) -> FileSource {
        FileSource { path }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}
impl Source for FileSource {
    fn name<'a>(&'a self) -> &'a OsStr {
        self.path.file_name().unwrap()
    }
    fn parse(&self, berg: &Berg) -> ParseResult {
        let mut errors = CompileErrors::new();
        let (expressions, metadata) = 
            if let Some(mut reader) = FileSourceReader::open(self, &mut errors, berg) {
                let expressions = Parser::new(&mut reader).parse();
                let metadata = reader.close();
                (expressions, metadata)
            } else {
                (vec![], SourceMetadata::new())
            };
        ParseResult { expressions, metadata, errors }        
    }
}

impl StringSource {
    pub fn new(name: String, contents: String) -> StringSource {
        StringSource { name, contents }
    }
    pub fn contents<'a>(&'a self) -> &'a String {
        &self.contents
    }
}
impl Source for StringSource {
    fn name<'a>(&'a self) -> &'a OsStr {
        String::as_ref(&self.name)
    }
    fn parse<'a>(&'a self, _: &Berg) -> ParseResult {
        let mut errors = CompileErrors::new();
        let (expressions, metadata) = {
            let mut reader = StringSourceReader::open(self, &mut errors);
            let expressions = Parser::new(&mut reader).parse();
            let metadata = reader.close();
            (expressions, metadata)
        };
        ParseResult { expressions, metadata, errors }
    }
}

