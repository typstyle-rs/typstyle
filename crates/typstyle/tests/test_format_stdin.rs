mod common;

use common::{typstyle_cmd_snapshot, Workspace};

const STDIN: &str = "#let  x  = (1+2)";

#[test]
fn test_nothing() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli(), @r"
    success: true
    exit_code: 0
    ----- stdout -----


    ----- stderr -----
    ");
}

#[test]
fn test_stdin() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().pass_stdin(STDIN), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let x = (1 + 2)

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_erroneous() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().pass_stdin("#"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #
    ----- stderr -----
    warn: Failed to parse stdin. The source is erroneous.
    ");
}

#[test]
fn test_stdin_column() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-c=0"]).pass_stdin(STDIN), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let x = (
      1
        + 2
    )

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_check() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--check"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    --- stdin
    +++ stdin
    @@ -1,1 +1,1 @@
    -#let  x  = (1+2)
    +#let x = (1 + 2)

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_inplace() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-i"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: cannot perform in-place formatting without at least one file being presented

    Usage: typstyle [OPTIONS] [INPUT]...

    For more information, try '--help'.
    ");
}

#[test]
fn test_stdin_inplace_check() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-i", "--check"]).pass_stdin(STDIN), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace [INPUT]...

    For more information, try '--help'.
    ");
}

#[test]
fn test_stdin_debug_ast() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["-a"]).pass_stdin(STDIN), @r##"
    success: true
    exit_code: 0
    ----- stdout -----
    Markup: 16 [
        Hash: "#",
        LetBinding: 15 [
            Let: "let",
            Space: "  ",
            Ident: "x",
            Space: "  ",
            Eq: "=",
            Space: " ",
            Parenthesized: 5 [
                LeftParen: "(",
                Binary: 3 [
                    Int: "1",
                    Plus: "+",
                    Int: "2",
                ],
                RightParen: ")",
            ],
        ],
    ]
    #let x = (1 + 2)

    ----- stderr -----
    "##);
}
