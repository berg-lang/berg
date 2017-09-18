use internals::*;

pub enum Source {
    File(PathBuf)
}

impl Source {
    pub fn from_arg(arg: String) -> Source {
        let mut path = env::current_dir().unwrap();
        path.push(arg);
        Source::File(path)
    }
    pub fn name(&self) -> &OsStr {
        match *self {
            Source::File(ref path) => path.file_name().unwrap(),
        }
    }
}

