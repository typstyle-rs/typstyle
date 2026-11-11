//! Integration tests for formatting selected line ranges with `--lines`.

mod common;

use common::{Workspace, typstyle_cmd_snapshot};
use insta_cmd::{Spawn, SpawnExt};

#[test]
fn test_stdin_multiple_ranges() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(
        space
            .cli()
            .args(["--lines=1:1", "--lines=3:3"])
            .pass_stdin("#let a  =  0\n#let b  =  1\n#let c  =  2\n"),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b  =  1
    #let c = 2

    ----- stderr -----
    "
    );
}

#[test]
fn test_stdin_overlapping_ranges() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(
        space
            .cli()
            .args(["--lines=1:1", "--lines=3:3"])
            .pass_stdin("#let x = (\n  1  +  2\n)\n#let y  =  3\n"),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let x = (
      1 + 2
    )
    #let y  =  3

    ----- stderr -----
    "
    );
}

#[test]
fn test_file_inplace_range() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["-i", "--lines", "1:1", "a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.read_string("a.typ"), "#let a = 0\n#let b  =  1\n");
}

#[test]
fn test_check_only_selected_lines() {
    let space = Workspace::new();
    let input = "#let a = 0\n#let b  =  1\n";

    typstyle_cmd_snapshot!(space.cli().args(["--check", "--lines=1:1"]).pass_stdin(input), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    typstyle_cmd_snapshot!(space.cli().args(["--check", "--lines=2:2"]).pass_stdin(input), @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    ");
}

#[test]
fn test_line_range_out_of_bounds() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--lines=1:3"]).pass_stdin("#let a = 0\n"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
      Cause: line range 1:3 is out of bounds; the input has 1 line
    ");
}

#[test]
fn test_out_of_bounds_wins_over_syntax_error() {
    let space = Workspace::new();
    let input = "#let a  =  1\n#let b  =  2\n#let c  =  3\n#let d  =  4; #\n";

    typstyle_cmd_snapshot!(space.cli().args(["--check", "--lines=50:50"]).pass_stdin(input), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
      Cause: line range 50:50 is out of bounds; the input has 4 lines
    ");

    typstyle_cmd_snapshot!(space.cli().args(["--check", "--lines=1:1"]).pass_stdin(input), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warn: Failed to parse stdin. The source is erroneous.
    ");
}

#[test]
fn test_pretty_doc_rejects_line_ranges() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(space.cli().args(["--lines=1:1", "--pretty-doc"]).pass_stdin("#let a = 0"), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--lines <FIRST:LAST>' cannot be used with '--pretty-doc'

    Usage: typstyle --lines <FIRST:LAST> [INPUT]...

    For more information, try '--help'.
    ");
}

#[test]
fn test_lf_line_boundaries() {
    let space = Workspace::new();
    let input = "one\u{2028}two\n#let b  =  1\n";

    let (_, output) = space
        .cli()
        .args(["--lines=2:2"])
        .pass_stdin(input.as_bytes())
        .spawn_with_info(None);
    assert!(output.status.success());
    assert_eq!(output.stdout, "one\u{2028}two\n#let b = 1\n".as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn test_syntax_error_leaves_input_unchanged() {
    let space = Workspace::new();

    typstyle_cmd_snapshot!(
        space
            .cli()
            .args(["--lines=1:1"])
            .pass_stdin("#let a  =  1\n#let b  =  2; #\n"),
        @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a  =  1
    #let b  =  2; #

    ----- stderr -----
    warn: Failed to parse stdin. The source is erroneous.
    "
    );
}
