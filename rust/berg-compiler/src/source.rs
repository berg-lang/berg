use berg::*;
use compile_errors::*;
use parser::Parser;
use source_reader::*;
use std::path::PathBuf;
use std::ffi::OsStr;

pub enum Source {
    File(PathBuf),
    Memory(Box<[u8]>),
}

pub fn main() {
    let source = Source::File("c:\\blah.txt");
    let x = open(&mut source);
    let y = open(source);
}

pub fn open(source: &mut       Source) {
    match source {
        Source::File(path) => {
            File::open(path)
        },
        Memory(mem) => {

        }
    }
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

impl<'a> SourceContext<'a> {
    pub fn new(berg: &'a Berg, source: &'a Source) -> SourceContext<'a> {
        SourceContext { berg, source, errors }
    }
    pub fn berg(&self) -> &'a Berg {
        self.berg
    }
    pub fn source(&self) -> &'a Source {
        self.source
    }
    pub fn report(error: CompileErrors) {
        errors.report()
    }
    pub fn close(self) -> CompileErrors {
        self.errors
    }
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
        let (expressions, char_data) = 
            if let Some(mut reader) = FileSourceReader::open(self, &mut errors, berg) {
                let expressions = Parser::new(&mut reader).parse();
                let char_data = reader.close();
                (expressions, char_data)
            } else {
                (vec![], CharData::new())
            };
        ParseResult { expressions, char_data, errors }        
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
        let (expressions, char_data) = {
            let mut reader = StringSourceReader::open(self, &mut errors);
            let expressions = Parser::new(&mut reader).parse();
            let char_data = reader.close();
            (expressions, char_data)
        };
        ParseResult { expressions, char_data, errors }
    }
}

