use internals::*;

pub mod line_column;
pub mod source_index;
pub mod source_metadata;

#[derive(Debug)]
pub enum Source {
    File(PathBuf),
    String(String, String),
}

impl Source {
    pub fn name<'a>(&'a self) -> &'a OsStr {
        match *self {
            Source::File(ref path) => path.file_name().unwrap(),
            Source::String(ref name, ..) => String::as_ref(name),
        }
    }
}
