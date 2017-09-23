use parser::internals::*;

#[derive(Debug)]
pub struct ParseResult<'a> {
    pub metadata: SourceMetadata<'a>,
    pub expressions: Vec<SyntaxExpression>,
    pub errors: CompileErrors,
}

impl<'a> ParseResult<'a> {
    pub fn source(&self) -> &'a Source {
        self.metadata.source()
    }
}

impl<'a> fmt::Display for ParseResult<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Source: {:?}", self.source().name())?;
        if self.expressions.len() > 0 {
            write!(f, "  Expressions:")?;
            for ref expression in &self.expressions {
                write!(f, "  - {}", expression)?;
            }
        }
        if self.errors.len() > 0 {
            write!(f, "  Errors:")?;
            for ref error in self.errors.all() {
                error.format(f, &self.metadata)?;
            }
        }
        Ok(())
    }
}
