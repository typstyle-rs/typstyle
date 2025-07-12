mod common;

use common::{typstyle_cmd_snapshot, Workspace};

#[test]
fn test_one() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.read_string("a.typ"), "#let a = 0\n");
}

#[test]
fn test_one_quiet() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "-q"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_erroneous() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let
    ----- stderr -----
    warn: Failed to parse a.typ. The source is erroneous.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_inplace_erroneous() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warn: Failed to parse a.typ. The source is erroneous.
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_one_check_quiet() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "--check", "-q"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: a.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_0() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b = 1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_1() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_2() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_0_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b = 1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert!(space.is_unmodified("b.typ"));
}

#[test]
fn test_two_1_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert_eq!(space.read_string("b.typ"), "#let b = 1\n");
}

#[test]
fn test_two_2_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert_eq!(space.read_string("a.typ"), "#let a = 0\n");
    assert_eq!(space.read_string("b.typ"), "#let b = 1\n");
}

#[test]
fn test_two_0_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b = 1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "--check"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_1_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: b.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_two_2_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a  =  0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: a.typ
    Would reformat: b.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_cwd() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");

    typstyle_cmd_snapshot!(space.cli().args(["./d/.."]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1
    #let c = 2

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_cwd_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");

    typstyle_cmd_snapshot!(space.cli().args(["./d/..", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: b.typ
    Would reformat: d/c.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_many() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");
    space.write_tracked("d/d/e.typ", "#let d  =  3\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "d"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let a = 0
    #let b = 1
    #let c = 2
    #let d = 3

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
}

#[test]
fn test_many_inplace() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");
    space.write_tracked("d/d/e.typ", "#let d  =  3\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "d", "-i"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    assert!(space.is_unmodified("a.typ"));
    assert!(space.is_modified("b.typ"));
    assert!(space.is_modified("d/c.typ"));
    assert!(space.is_modified("d/d/e.typ"));
}

#[test]
fn test_many_check() {
    let mut space = Workspace::new();
    space.write_tracked("a.typ", "#let a = 0\n");
    space.write_tracked("b.typ", "#let b  =  1\n");
    space.write_tracked("d/c.typ", "#let c  =  2\n");
    space.write_tracked("d/d/e.typ", "#let d  =  3\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "b.typ", "d", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: b.typ
    Would reformat: d/c.typ
    Would reformat: d/d/e.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_diff_single_line() {
    let mut space = Workspace::new();
    space.write_tracked("single.typ", "#let x=1+2");

    typstyle_cmd_snapshot!(space.cli().args(["single.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: single.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}

#[test]
fn test_diff_multiline() {
    let mut space = Workspace::new();
    space.write_tracked("multi.typ", "#let x=1\n#let y=2+3");

    typstyle_cmd_snapshot!(space.cli().args(["multi.typ", "--check"]), @r"
    success: false
    exit_code: 1
    ----- stdout -----
    Would reformat: multi.typ

    ----- stderr -----
    ");

    assert!(space.all_unmodified());
}
