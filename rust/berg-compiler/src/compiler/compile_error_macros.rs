use compiler::Compiler;
use compiler::source_data::{ByteRange,SourceIndex};
use std::str;

#[macro_export]
macro_rules! compile_errors {
    ($(pub struct $name:ident $fields:tt ($code:expr) = $message_type:ident $message:tt;)*) => {
        $(compile_errors! { @single pub struct $name $fields ($code) = $message_type $message; })*
    };
    (@single pub struct $name:ident $fields:tt ($code:expr) = $message_type:ident $message:tt;) => {
        compile_errors! { @define_struct $name, $fields, $message_type }
        compile_errors! { @impl_struct $name, $code, $fields, $message_type, $message }
    };
    (@define_struct $name:ident, { $(pub $field:tt: $field_type:ty),* }, string_generic) => {
        #[derive(Debug,Clone)]
        pub struct $name { $(pub $field: $field_type,)* }
    };
    (@define_struct $name:ident, { $(pub $field:tt: $field_type:ty),* }, format_generic) => {
        #[derive(Debug,Clone)]
        pub struct $name { $(pub $field: $field_type,)* }
    };
    (@define_struct $name:ident, { $(pub $field:tt: $field_type:ty),* }, $message_type:ident) => {
        #[derive(Debug,Clone)]
        pub struct $name { pub source: SourceIndex, $(pub $field: $field_type,)* }
    };
    (@impl_struct $name:ident, $code:expr, $fields:tt, $message_type:ident, $message:tt) => {
        impl $name {
            pub const CODE: u32 = $code;
        }
        impl CompileError for $name {
            fn code(&self) -> u32 { $name::CODE }
            compile_errors! { @message $message_type, $message, $fields }
        }
    };
    (@message string, ($range:tt, $message:tt), $fields:tt) => (
        fn message(&self, _: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: $message.to_string(),
                location: CompileErrorLocation::SourceRange { source: self.source, range: self.$range.clone() }
            }
        }
    );
    (@message string_source, ($message:tt), $fields:tt) => (
        fn message(&self, _: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: $message.to_string(),
                location: CompileErrorLocation::SourceOnly { source: self.source }
            }
        }
    );
    (@message string_generic, ($message:tt), $fields:tt) => (
        fn message(&self, _: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: $message.to_string(),
                location: CompileErrorLocation::Generic { }
            }
        }
    );
    (@message format, ($range:tt, $message:tt), { $(pub $field:tt: $field_type:tt),* }) => (
        fn message(&self, _compiler: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: format!($message, $($field = compile_errors!(@field_value _compiler, self, (self.$field), $field_type)),*),
                location: CompileErrorLocation::SourceRange { source: self.source, range: self.$range.clone() },
            }
        }
    );
    (@message format_source, ($message:tt), { $(pub $field:tt: $field_type:tt),* }) => (
        fn message(&self, _compiler: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: format!($message, $($field = compile_errors!(@field_value _compiler, self, (self.$field), $field_type)),*),
                location: CompileErrorLocation::SourceOnly { source: self.source },
            }
        }
    );
    (@message format_generic, ($message:tt), { $(pub $field:tt: $field_type:tt),* }) => (
        fn message(&self, compiler: &Compiler) -> CompileErrorMessage {
            CompileErrorMessage {
                message: format!($message, $($field = compile_errors!(@field_value compiler, self, (self.$field), $field_type)),*),
                location: CompileErrorLocation::Generic,
            }
        }
    );
    (@field_value $compiler:ident, $self:ident, $value:tt, ByteRange) => ({
        use compiler::compile_error_macros::source_string;
        source_string($compiler, $self.source, &$value)
    });
    (@field_value $compiler:ident, $self:ident, $value:tt, $type:tt) => ($value);
}

pub fn source_string(compiler: &Compiler, source: SourceIndex, range: &ByteRange) -> String {
    let buffer = compiler.with_source(source, |source_data| source_data.source_spec().open(compiler, source));
    if range.end <= buffer.len() {
        if let Ok(string) = str::from_utf8(&buffer[range]) {
            return string.to_string();
        }
    }
    String::from("ERROR: source may have changed since compiling, source range is no longer valid UTF-8")
}

