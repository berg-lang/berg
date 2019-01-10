use eval::{ExpressionTreeFormatter, RootRef};
use error::{BergError, BergResult, EvalResult};
use parser;
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use std::ops::Range;
use std::rc::Rc;
use std::u32;
use syntax::{AstData, AstRef};
use util::indexed_vec::{to_indexed_cow, IndexedSlice};
use util::try_from::TryFrom;
use util::type_name::TypeName;
use value::BergVal;

index_type! {
    pub struct ByteIndex(pub u32) with Display,Debug <= u32::MAX;
}
pub type ByteSlice = IndexedSlice<u8, ByteIndex>;
pub type ByteRange = Range<ByteIndex>;

#[derive(Clone)]
pub struct SourceRef<'a>(Rc<SourceData<'a>>);

impl<'a> fmt::Debug for SourceRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self.0 {
            SourceData::File(ref path, ..) => write!(f, "File({:?})", path),
            SourceData::Memory(name, ..) => write!(f, "Memory({:?})", name),
        }
    }
}

#[derive(Debug)]
enum SourceData<'a> {
    File(Cow<'a, Path>, RootRef),
    Memory(&'a str, &'a [u8], RootRef),
}

impl<'a> SourceRef<'a> {
    pub fn file(path: Cow<'a, Path>, root: RootRef) -> Self {
        SourceRef(Rc::new(SourceData::File(path, root)))
    }
    pub fn memory(name: &'a str, bytes: &'a [u8], root: RootRef) -> Self {
        SourceRef(Rc::new(SourceData::Memory(name, bytes, root)))
    }
    pub fn root(&self) -> &RootRef {
        match *self.0 {
            SourceData::File(_, ref root) | SourceData::Memory(_, _, ref root) => root,
        }
    }
    pub fn parse(&self) -> AstRef<'a> {
        let parsed = parser::parse(self);
        println!();
        println!("Parsed:");
        print!(
            "{}",
            ExpressionTreeFormatter(parsed.expression(), &parsed, 1)
        );
        parsed
    }
    pub fn evaluate(self) -> BergResult<'a> {
        self.parse().evaluate()
    }
    pub fn evaluate_to<T: TypeName + TryFrom<BergVal<'a>, Error = BergVal<'a>>>(
        self,
    ) -> BergResult<'a, T> {
        self.parse().evaluate_to()
    }

    pub fn name(&'a self) -> Cow<'a, str> {
        match *self.0 {
            SourceData::File(ref path, ..) => path.to_string_lossy(),
            SourceData::Memory(name, ..) => name.into(),
        }
    }
    pub fn absolute_path<'p>(&'p self) -> EvalResult<'a, Cow<'p, Path>>
    where
        'a: 'p,
    {
        match *self.0 {
            SourceData::File(ref path, ref root) => absolute_path(path.clone(), root),
            SourceData::Memory(..) => unreachable!(),
        }
    }
    pub fn open(&self, ast: &mut AstData<'a>) -> Cow<'a, ByteSlice> {
        match *self.0 {
            SourceData::File(ref path, ref root) => open_file_and_report(path.clone(), ast, root),
            SourceData::Memory(_, buffer, _) => open_memory_and_report(buffer, ast),
        }
    }
}

fn open_file_and_report<'a>(
    path: Cow<Path>,
    ast: &mut AstData<'a>,
    root: &RootRef,
) -> Cow<'static, ByteSlice> {
    let result: Cow<[u8]> = match absolute_path(path, root) {
        Ok(ref path) => open_file(path, ast),
        Err(_) => {
            ast.file_open_error = Some((
                BergError::CurrentDirectoryError,
                io::Error::new(io::ErrorKind::Other, "current directory error: you should NOT be seeing this error! You should see root.root_path's error instead."),
            ));
            Cow::Borrowed(&[])
        }
    };
    let size = result.len();
    if size >= usize::from(ByteIndex::MAX) && ast.file_open_error.is_none() {
        ast.file_open_error = Some((
            BergError::SourceTooLarge(size),
            io::Error::new(io::ErrorKind::Other, "source too large"),
        ));
    }
    to_indexed_cow(result)
}

fn open_file<'a>(path: &Path, ast: &mut AstData<'a>) -> Cow<'static, [u8]> {
    match File::open(path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            if let Err(io_error) = file.read_to_end(&mut buffer) {
                ast.file_open_error = Some((BergError::IoReadError, io_error));
            }
            Cow::Owned(buffer)
        }
        Err(io_error) => {
            let error = match io_error.kind() {
                io::ErrorKind::NotFound => BergError::SourceNotFound,
                _ => BergError::IoOpenError,
            };
            ast.file_open_error = Some((error, io_error));
            Cow::Borrowed(&[])
        }
    }
}

fn absolute_path<'a>(
    path: Cow<'a, Path>,
    root: &RootRef,
) -> EvalResult<'a, Cow<'a, Path>> {
    if path.is_relative() {
        match *root.root_path() {
            Ok(ref root_path) => Ok(Cow::Owned(root_path.join(path))),
            Err(_) => BergError::CurrentDirectoryError.err(),
        }
    } else {
        Ok(path)
    }
}

fn open_memory_and_report<'a>(buffer: &'a [u8], ast: &mut AstData<'a>) -> Cow<'a, ByteSlice> {
    let size = buffer.len();
    if size >= usize::from(ByteIndex::MAX) && ast.file_open_error.is_none() {
        ast.file_open_error = Some((
            BergError::SourceTooLarge(size),
            io::Error::new(io::ErrorKind::Other, "source too large"),
        ));
    }
    to_indexed_cow(Cow::Borrowed(buffer))
}

impl<'a> fmt::Display for SourceRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
