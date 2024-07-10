use crate::*;

#[test]
fn delimited_block_h1() {
    expect(
        "
X:
    H1
    ===
    10
X.H1
",
    )
    .to_yield(10)
}

#[test]
fn delimited_block_h2() {
    expect(
        "
X:
    H2
    ---
    20
X.H2
",
    )
    .to_yield(20)
}

#[test]
fn delimited_block_h1_h2() {
    expect(
        "
X:
    H1
    ===
    H2
    ---
    20
X.H1.H2
",
    )
    .to_yield(20)
}

#[test]
#[ignore = "Not working yet"]
fn delimited_block_h1_h1() {
    expect(
        "
X:
    H1A
    ===
    10
    H1B
    ===
    20
X.H1B
",
    )
    .to_yield(20)
}
