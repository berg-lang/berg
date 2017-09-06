use std::io;
use std::env;
use std::fs;
use std::ffi::OsStr;
use std::path::PathBuf;
use source::Source::*;

pub enum Source {
    File(FileSource)
}

impl Source {
    pub fn from_env(arg: String) -> Source {
        let mut path = env::current_dir().unwrap();
        path.push(arg);
        File(FileSource { path })
    }
    pub fn name(&self) -> &OsStr {
        match *self {
            File(ref f) => f.path.file_name().unwrap(),
        }
    }
}

pub struct FileSource {
    path: PathBuf
}

impl FileSource {
    pub fn open(&self) -> io::Result<io::BufReader<fs::File>> {
        let file = fs::File::open(&self.path)?;
        let buf = io::BufReader::new(file);
        Ok(buf)
    }
}