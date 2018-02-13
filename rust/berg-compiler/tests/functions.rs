#[macro_use]
pub mod compiler_test;

compiler_tests! {
    function: "Square: :X*X; Square 10" => value(100),
    function_result: "F: { X: 1; Y: 2; X+Y }; F" => value(3),
    function_result_add: "F: { X: 1; Y: 2; X+Y }; F + 2" => value(5),
    function_field: "F: { X: 1; Y: 2; X+Y }; F.X" => value(1),
    function_result_and_field: "F: { X: 1; Y: 2; X+Y }; F + F.X" => value(4),
}
