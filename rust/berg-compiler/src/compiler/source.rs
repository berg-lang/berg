use std::path::PathBuf;
use std::ffi::OsStr;

#[derive(Debug)]
pub enum Source {
    File(PathBuf),
    Memory(String, Vec<u8>),
}

impl Source {
    pub fn file<P: Into<PathBuf>>(path: P) -> Source {
        Source::File(path.into())
    }
    pub fn memory<S: Into<String>, B: Into<Vec<u8>>>(name: S, contents: B) -> Source {
        Source::Memory(name.into(), contents.into())
    }
    pub fn name<'a>(&'a self) -> &'a OsStr {
        match self {
            &Source::File(ref path) => path.as_ref(),
            &Source::Memory(ref name, ..) => name.as_ref(),
        }
    }
}

