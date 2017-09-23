use internals::*;
use parser;

pub struct Berg {
    root: Result<PathBuf>,
}

impl Berg {
    pub fn from_env() -> Berg {
        Berg { root: env::current_dir() }
    }
    pub fn new(root: PathBuf) -> Berg {
        Berg { root: Ok(root) }
    }

    pub fn root(&self) -> &Result<PathBuf> {
        &self.root
    }

    pub fn file(&self, path: PathBuf) -> Source {
        Source::File(path)
    }
    pub fn string(&self, name: String, contents: String) -> Source {
        Source::String(name, contents)
    }
    pub fn parse<'a>(&self, source: &'a Source) -> ParseResult<'a> {
        let berg = Berg::from_env();
        parser::parse(source, &berg)
    }
}
