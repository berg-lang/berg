//!
//! Keywords are fields in the root. When the identifier `true` is in the code,
//! it's treated as a normal variable reference and looked up in scope (which
//! includes the root scope).
//!

#[allow(clippy::upper_case_acronyms)]
fields! { TRUE, FALSE, IF, ELSE, WHILE, FOREACH, BREAK, CONTINUE, TRY, CATCH, FINALLY, THROW, }

pub fn identifiers(&self) -> StringInterner<StringBackend<IdentifierIndex>> {
    identifiers::intern_all()
}
