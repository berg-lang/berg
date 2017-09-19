use parser::internals::*;

pub struct ParseResult<'s> {
    pub source: &'s Source,
    pub metadata: SourceMetadata,
    pub expressions: Vec<SyntaxExpression>,
    pub errors: CompileErrors,
}

impl<'s> fmt::Display for ParseResult<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Source: {:?}", self.source.name())?;
        if self.expressions.len() > 0 {
            write!(f, "  Expressions:")?;
            for ref expression in &self.expressions {
                write!(f, "  - {}", expression)?;
            }
        }
        if self.errors.len() > 0 {
            write!(f, "  Errors:")?;
            for ref error in self.errors.all() {
                error.format(f, &self)?;
            }
        }
        Ok(())
    }
}
