use crate::*;

#[test]
fn zero_indent_file() {
    expect(
        "
:x = 10
x
",
    )
    .to_yield(10)
}

#[test]
fn fully_indented_file() {
    expect(
        "
    :x = 10
    x
",
    )
    .to_yield(10)
}

#[test]
fn block() {
    expect(
        "
{
    :x = 10
    x
}
    ",
    )
    .to_yield(10)
}

#[test]
fn block_with_undented_close() {
    expect(
        "
    {
        :x = 10
        x
}
    ",
    )
    .to_error(CloseWithoutOpen, "}");
}

#[test]
fn block_with_indented_close() {
    expect(
        "
{
    :x = 10
    x
    }
    ",
    )
    .to_error(OpenWithoutClose, "{");
}

#[test]
fn nested_block_with_missing_indented_close() {
    expect(
        "
{
    {
        :x = 10
        x
}
    ",
    )
    .to_error(OpenWithoutClose, "{".after("{"));
}

#[test]
fn nested_block_with_missing_indented_open_ought_to_report_the_inner_one_but_berg_does_it_wrong() {
    expect(
        "
{
        :x = 10
        x
    }
}
    ",
    )
    .to_error(CloseWithoutOpen, "}".after("}"));
}

#[test]
fn parens() {
    expect(
        "
(
    :x = 10
    x
)
    ",
    )
    .to_yield(10)
}

#[test]
fn parens_with_undented_close() {
    expect(
        "
    (
        :x = 10
        x
)
    ",
    )
    .to_error(CloseWithoutOpen, ")");
}

#[test]
fn parens_with_indented_close() {
    expect(
        "
(
    :x = 10
    x
    )
    ",
    )
    .to_error(OpenWithoutClose, "(");
}

#[test]
fn nested_parens_with_missing_indented_close() {
    expect(
        "
(
    (
        :x = 10
        x
)
    ",
    )
    .to_error(OpenWithoutClose, "(".after("("));
}

#[test]
fn nested_parens_with_missing_indented_open_ought_to_report_the_inner_one_but_berg_does_it_wrong() {
    expect(
        "
(
        :x = 10
        x
    )
)
    ",
    )
    .to_error(CloseWithoutOpen, ")".after(")"));
}
