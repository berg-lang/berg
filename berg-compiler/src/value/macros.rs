#[macro_export]
macro_rules! tuple {
    ( $( $x:tt ),* ) => { { use berg_compiler::BergVal; BergVal::from_iter(vec![ $( val!($x) ),* ].drain(..)) } };
}

#[macro_export]
macro_rules! val {
    ( [ $( $x:tt ),* ] ) => { tuple!( $( $x ),* ) };
    ( ( $( $x:tt ),+ ) ) => { tuple!( $( $x ),+ ) };
    ( ( $x:expr ) ) => { val!($x) };
    ( $x:expr ) => { { use berg_compiler::BergVal; BergVal::from($x) } };
}