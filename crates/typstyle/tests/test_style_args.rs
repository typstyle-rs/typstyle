mod common;

use common::{Workspace, typstyle_cmd_snapshot};

#[test]
fn test_tab_width() {
    let space = Workspace::new();

    let stdin = "#let f(x) = {
for i in range(0, 5) {
     x = x + i
 }
}";

    typstyle_cmd_snapshot!(space.cli().args(["-t=4"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
        for i in range(0, 5) {
            x = x + i
        }
    }

    ----- stderr -----
    ");
}

#[test]
fn test_reorder_import_items() {
    let space = Workspace::new();

    let stdin = r#"#import "module.typ": xyz, func as renamed, h.i.j, a.b.c"#;

    typstyle_cmd_snapshot!(space.cli().pass_stdin(stdin), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "module.typ": a.b.c, func as renamed, h.i.j, xyz

    ----- stderr -----
    "#);
    typstyle_cmd_snapshot!(space.cli().args(["--no-reorder-import-items"]).pass_stdin(stdin), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "module.typ": xyz, func as renamed, h.i.j, a.b.c

    ----- stderr -----
    "#);
}

#[test]
fn test_wrap_text() {
    let space = Workspace::new();

    let stdin = "lorem  ipsum   dolor sit amet, consectetur   adipiscing elit.";

    typstyle_cmd_snapshot!(space.cli().args(["-c=20", "--wrap-text"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    lorem ipsum dolor
    sit amet,
    consectetur
    adipiscing elit.

    ----- stderr -----
    ");
}
