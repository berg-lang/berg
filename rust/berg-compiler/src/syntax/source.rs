use crate::error::{BergError, EvalResult};
use crate::eval::RootRef;
use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::io::Read;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use std::u32;
use crate::util::indexed_vec::{to_indexed_cow, IndexedSlice};

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

#[derive(Debug)]
pub struct SourceOpenError<'a>(pub BergError<'a>, pub io::Error);

#[derive(Debug)]
pub struct SourceBuffer<'a> {
    pub buffer: Cow<'a, ByteSlice>,
    pub source_open_error: Option<SourceOpenError<'a>>,
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
    pub fn open(&self) -> SourceBuffer<'a> {
        match *self.0 {
            SourceData::File(ref path, ref root) => self.open_file(path.clone(), root),
            SourceData::Memory(_, buffer, _) => self.open_memory(buffer),
        }
    }

    fn open_file(&self, path: Cow<Path>, root: &RootRef) -> SourceBuffer<'a> {
        // Grab the path relative to the root, so that we can open it.
        let path = match absolute_path(path, root) {
            Ok(path) => path,
            Err(_) => return SourceBuffer::error(SourceOpenError::new(BergError::CurrentDirectoryError, io::ErrorKind::Other, "current directory error: you should NOT be seeing this error! You should see root.root_path's error instead.")),
        };

        // Open the file.
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(io_error) => {
                let error = match io_error.kind() {
                    io::ErrorKind::NotFound => BergError::SourceNotFound,
                    _ => BergError::IoOpenError,
                };
                return SourceBuffer::from_slice(&[], Some(SourceOpenError(error, io_error)));
            }
        };

        // Read the file, and check if the resulting buffer is too large.
        let mut buffer = Vec::new();
        let source_open_error = match file.read_to_end(&mut buffer) {
            Ok(_) => check_source_too_large(buffer.len()),
            Err(io_error) => Some(SourceOpenError(BergError::IoReadError, io_error)),
        };
        SourceBuffer::from_cow(Cow::Owned(buffer), source_open_error)
    }

    fn open_memory(&self, buffer: &'a [u8]) -> SourceBuffer<'a> {
        SourceBuffer::from_slice(buffer, check_source_too_large(buffer.len()))
    }
}

fn check_source_too_large<'a>(size: usize) -> Option<SourceOpenError<'a>> {
    if size < usize::from(ByteIndex::MAX) {
        None
    } else {
        Some(SourceOpenError::new(
            BergError::SourceTooLarge(size),
            io::ErrorKind::Other,
            "source too large",
        ))
    }
}

fn absolute_path<'a>(path: Cow<'a, Path>, root: &RootRef) -> EvalResult<'a, Cow<'a, Path>> {
    if path.is_relative() {
        match *root.root_path() {
            Ok(ref root_path) => Ok(Cow::Owned(root_path.join(path))),
            Err(_) => BergError::CurrentDirectoryError.err(),
        }
    } else {
        Ok(path)
    }
}

impl<'a> fmt::Display for SourceRef<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<'a> SourceOpenError<'a> {
    pub fn new(error: BergError<'a>, io_error_kind: io::ErrorKind, io_error_message: &str) -> Self {
        SourceOpenError(error, io::Error::new(io_error_kind, io_error_message))
    }
}

impl<'a> SourceBuffer<'a> {
    fn from_cow(buffer: Cow<'a, [u8]>, source_open_error: Option<SourceOpenError<'a>>) -> Self {
        SourceBuffer {
            buffer: to_indexed_cow(buffer),
            source_open_error,
        }
    }
    fn from_slice(buffer: &'a [u8], source_open_error: Option<SourceOpenError<'a>>) -> Self {
        SourceBuffer::from_cow(Cow::Borrowed(buffer), source_open_error)
    }
    fn error(source_open_error: SourceOpenError<'a>) -> Self {
        SourceBuffer::from_slice(&[], Some(source_open_error))
    }
}
