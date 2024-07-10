use crate::*;

#[test]
fn indented_trail() {
    expect(
        "
1 + 2 *
    3 + 4
    ",
    )
    .to_yield(1 + 2 * (3 + 4))
}

#[test]
#[ignore = "Treats it as a single line"]
fn nonindented_trail() {
    expect(
        "
1 + 2 *
3 + 4
    ",
    )
    .to_yield(1 + 2 * (3 + 4))
}

#[test]
fn undented_trail_error() {
    expect(
        "
    1 + 2 *
3 + 4
    ",
    )
    .to_error(MissingOperand, "*")
}

#[test]
#[ignore = "Treats 3+4 as a single line but not 1+2"]
fn indented_lead() {
    expect(
        "
1 + 2
    * 3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

// TODO why is this different from the indented continued expression? Something seems fishy.
#[test]
#[ignore = "Treats it as a single line"]
fn nonindented_lead() {
    expect(
        "
1 + 2
* 3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
fn undented_lead() {
    expect(
        "
    1 + 2
* 3 + 4
    ",
    )
    .to_yield((1 + 2) * 3 + 4)
}

#[test]
#[ignore = "Groups 3+4 but not 1+2"]
fn indented_bare_left() {
    expect(
        "
1 + 2
*
    3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Groups 3+4 but not 1+2"]
fn indented_bare_right() {
    expect(
        "
1 + 2
    *
    3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Groups 3+4 but not 1+2"]
fn indented_bare_mid() {
    expect(
        "
1 + 2
  *
    3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Treats as a single line"]
fn nonindented_bare() {
    expect(
        "
1 + 2
*
3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
fn nonindented_bare_left() {
    expect(
        "
    1 + 2
*
    3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Treats it as a single line"]
fn nonindented_bare_right() {
    expect(
        "
1 + 2
    *
3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Groups 1+2 but not 3+4"]
fn undented_bare_left() {
    expect(
        "
    1 + 2
*
3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Groups 1+2 but not 3+4"]
fn undented_bare_mid() {
    expect(
        "
    1 + 2
  *
3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

#[test]
#[ignore = "Makes no sense that this errors but undented_bare_mid does not"]
fn undented_bare_right() {
    expect(
        "
    1 + 2
    *
3 + 4
    ",
    )
    .to_yield((1 + 2) * (3 + 4))
}

mod sandwich {
    use crate::*;

    #[test]
    #[ignore = "Treats 3+4*5+6 as a single line. Also does not treat 1+2 as a single line."]
    fn trail_lead() {
        expect(
            "
    1 + 2 *
        3 + 4
    * 5 + 6
        ",
        )
        .to_yield((1 + 2) * (3 + 4) * (5 + 6))
    }

    #[test]
    #[ignore = "Treats 3+4 as a single line but not the others"]
    fn lead_lead() {
        expect(
            "
    1 + 2
        * 3 + 4
    * 5 + 6
        ",
        )
        .to_yield((1 + 2) * (3 + 4) * (5 + 6))
    }

    #[test]
    fn trail_trail() {
        expect(
            "
    1 + 2 *
        3 + 4 *
    5 + 6
        ",
        )
        .to_error(MissingOperand, "*".after("*"))
    }

    #[test]
    fn lead_trail() {
        expect(
            "
    1 + 2
        * 3 + 4 *
    5 + 6
        ",
        )
        .to_error(MissingOperand, "*".after("*"))
    }

    #[test]
    #[ignore = "Treats 3+4*5+6 as a single line. Also does not treat 1+2 or 7+8 as single lines."]
    fn multiline() {
        expect(
            "
    1 + 2 *
        3 + 4 * 
        5 + 6
    * 7 + 8
        ",
        )
        .to_yield((1 + 2) * (3 + 4) * (5 + 6) * (7 + 8))
    }
}
