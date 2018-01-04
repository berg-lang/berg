#[macro_export]
macro_rules! compile_errors {
    ($(pub struct $name:ident $fields:tt ($code:expr) = $location_type:ident($($location:tt)*):$message_type:ident($($message:tt)*);)*) => {
        $(compile_errors! { @single pub struct $name $fields ($code) = $location_type($($location)*):$message_type($($message)*); })*

        #[derive(Debug,Copy,Clone,PartialEq)]
        pub enum CompileErrorCode {
            $($name,)*
        }

        use std::fmt::*;
        impl Display for CompileErrorCode {
            fn fmt(&self, f: &mut Formatter) -> Result {
                match *self {
                    $(CompileErrorCode::$name => write!(f, "{}", stringify!($name)),)*
                }
            }
        }
    };
    (@single pub struct $name:ident $fields:tt ($code:expr) = $location_type:ident($($location:tt)*):$message_type:ident($($message:tt)*);) => {
        compile_errors! { @define_struct pub struct $name $fields }
        compile_errors! { @impl_struct pub struct $name $fields ($code) = $location_type($($location)*):$message_type($($message)*) }
    };
    (@define_struct pub struct $name:ident { $(pub $field:tt: $field_type:ty),* }) => {
        #[derive(Debug,Clone,PartialEq)]
        pub struct $name { $(pub $field: $field_type,)* }
    };
    (@impl_struct pub struct $name:ident $fields:tt ($code:expr) = $location_type:ident($($location:ident)*):$message_type:ident($($message:tt)*)) => {
        compile_errors! { @new pub struct $name $fields }
        impl CompileError for $name {
            fn location(&self) -> CompileErrorLocation { compile_errors! { @location self, $location_type($($location)*) } }
            fn code(&self) -> CompileErrorCode { CompileErrorCode::$name }
            #[allow(unused_variables)]
            fn message<'c>(&self, compiler: &'c Compiler) -> CompileErrorMessage {
                CompileErrorMessage {
                    message: compile_errors!(@message $message_type($($message)*), self, compiler, $fields),
                    location: self.location()
                }
            }
            fn box_clone(&self) -> Box<CompileError> { Box::new(self.clone()) }
        }
    };
    (@new pub struct $name:ident { $(pub $field:tt: $field_type:ty),* }) => {
        impl $name {
            pub fn new($($field: $field_type),*) -> $name {
                $name { $($field: $field.into(),)* }
            }
            pub fn value($($field: $field_type),*) -> Value {
                Self::new($($field),*).into()
            }
        }
    };
    (@location $self:ident, range($range:ident)) => (CompileErrorLocation::SourceRange($self.$range.clone()));
    (@location $self:ident, source($source:ident)) => (CompileErrorLocation::SourceOnly($self.$source));
    (@location $self:ident, generic()) => (CompileErrorLocation::Generic);
    (@message string($message:tt), $self:ident, $compiler:ident, $fields:tt) => ($message.to_string());
    (@message format($message:tt), $self:ident, $compiler:ident, { $(pub $field:tt: $field_type:ty),* }) => (format!($message, $($field = $self.$field.disp($compiler)),*));
}
