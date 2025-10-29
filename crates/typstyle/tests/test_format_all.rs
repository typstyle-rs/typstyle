mod common;

use common::{Workspace, typstyle_cmd_snapshot};

#[test]
fn test_all_0() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");
    space.write_tracked("x/b.typ", "#let b  =  1");
    space.write_tracked("x/y/.c.typ", "#let c  =  2");
    space.write_tracked("x/.z/d.typ", "#let d  =  3");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "-v"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully formatted 2 files (0 unchanged) in [DURATION]

    ----- stderr -----
    ");

    assert_eq!(space.read_string("a.typ"), "#let a = 0\n");
    assert_eq!(space.read_string("x/b.typ"), "#let b = 1\n");
    assert!(space.is_unmodified("x/y/.c.typ"));
    assert!(space.is_unmodified("x/.z/d.typ"));
}

#[test]
fn test_all_1() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("x/b.typ", "#let b  =  1");
    space.write_tracked("x/y/.c.typ", "#let c  =  2");
    space.write_tracked("x/.z/d.typ", "#let d  =  3");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "-v"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully formatted 1 file (1 unchanged) in [DURATION]

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert_eq!(space.read_string("x/b.typ"), "#let b = 1\n");
    assert!(space.is_unmodified("x/y/.c.typ"));
    assert!(space.is_unmodified("x/.z/d.typ"));
}

#[test]
fn test_all_2() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("x/b.typ", "#let b = 1\n");
    space.write_tracked("x/y/.c.typ", "#let c  =  2");
    space.write_tracked("x/.z/d.typ", "#let d  =  3");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "-v"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully formatted 0 file (2 unchanged) in [DURATION]

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert!(space.is_unmodified("x/b.typ"));
    assert!(space.is_unmodified("x/y/.c.typ"));
    assert!(space.is_unmodified("x/.z/d.typ"));
}

#[test]
fn test_all_0_check() {
    // NOTE - output order is undefined
    let mut space = Workspace::new();
    // space.write_tracked("a.typ", "#let a  =  0");
    space.write_tracked("x/b.typ", "#let b  =  1");
    space.write_tracked("x/y/.c.typ", "#let c  =  2");
    space.write_tracked("x/.z/d.typ", "#let d  =  3");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "--check", "-v"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace --verbose <INPUT>...

    For more information, try '--help'.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_all_1_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("x/b.typ", "#let b  =  1");
    space.write_tracked("x/y/.c.typ", "#let c  =  2");
    space.write_tracked("x/.z/d.typ", "#let d  =  3");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "--check", "-v"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace --verbose <INPUT>...

    For more information, try '--help'.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_all_2_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("x/b.typ", "#let b = 1\n");
    space.write_tracked("x/y/.c.typ", "#let c  =  2");
    space.write_tracked("x/.z/d.typ", "#let d  =  3");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "--check", "-v"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace --verbose <INPUT>...

    For more information, try '--help'.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_all_erroneous() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("x/b.typ", "#let b  =  1");
    space.write_tracked("x/y/c.typ", "#let c  =  2; #");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "-v"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully formatted 1 file (2 unchanged) in [DURATION]

    ----- stderr -----
    warn: Failed to parse x/y/c.typ. The source is erroneous.
    ");

    assert!(space.is_unmodified("a.typ"));
    assert_eq!(space.read_string("x/b.typ"), "#let b = 1\n");
    assert!(space.is_unmodified("x/y/c.typ"));
}

#[test]
fn test_all_erroneous_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("x/b.typ", "#let b  =  1");
    space.write_tracked("x/y/c.typ", "#let c  =  2; #");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "--check", "-v"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace --verbose <INPUT>...

    For more information, try '--help'.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_all_column() {
    let space = Workspace::new();
    space.write("a.typ", "#let a  =  (1 + 2)");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "-c=0", "-v"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Successfully formatted 1 file (0 unchanged) in [DURATION]

    ----- stderr -----
    ");

    assert_eq!(
        space.read_string("a.typ"),
        "#let a = (
  1
    + 2
)
"
    );
}

#[test]
fn test_dir_all_check() {
    let space = Workspace::new();
    space.write("a.typ", "#let a  =  0");
    space.write("x/b.typ", "#let b  =  1");
    space.write("x/y/.c.typ", "#let c  =  2");

    typstyle_cmd_snapshot!(space.cli().args([".", "-i", "x", "--check", "-v"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the argument '--inplace' cannot be used with '--check'

    Usage: typstyle --inplace --verbose <INPUT>...

    For more information, try '--help'.
    ");
}
