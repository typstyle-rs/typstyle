//! Integration tests for `typstyle.toml` configuration files: discovery,
//! precedence against CLI options, `--config`, `--isolated`, and
//! `--stdin-filename`.

mod common;

use common::{Workspace, typstyle_cmd_snapshot};

#[test]
fn test_config_file_from_parent_dir() {
    let mut space = Workspace::new();
    space.write("typstyle.toml", "indent-width = 4\n");
    space.write_tracked(
        "nested/a.typ",
        "#let f(x) = {\nfor i in range(0, 5) {\n     x = x + i\n }\n}",
    );

    typstyle_cmd_snapshot!(space.cli().args(["nested/a.typ"]), @r"
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

    assert!(space.all_unmodified());
}

#[test]
fn test_nearest_config_file_replaces_parent_config() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-width = 20\nindent-width = 4\n");
    space.write("nested/typstyle.toml", "indent-width = 8\n");
    space.write("nested/a.typ", "#let f(x) = {\nx\n}");

    // The nested file replaces the parent one: its missing `line-width` falls
    // back to the default of 80 rather than inheriting the parent's 20.
    typstyle_cmd_snapshot!(space.cli().args(["nested/a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
            x
    }

    ----- stderr -----
    ");
}

#[test]
fn test_config_file_for_stdin() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-width = 20\nwrap-mode = \"fill\"\n");
    let stdin = "lorem  ipsum   dolor sit amet, consectetur   adipiscing elit.";

    typstyle_cmd_snapshot!(space.cli().pass_stdin(stdin), @r"
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

#[test]
fn test_cli_options_override_config_file() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-width = 20\nwrap-mode = \"fill\"\n");
    let stdin = "lorem  ipsum   dolor sit amet, consectetur   adipiscing elit.";

    // The explicit width wins; the configured wrap mode still applies.
    typstyle_cmd_snapshot!(space.cli().args(["--line-width=80"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    lorem ipsum dolor sit amet, consectetur adipiscing elit.

    ----- stderr -----
    ");
}

#[test]
fn test_missing_cli_options_keep_config_file_values() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-width = 20\nwrap-mode = \"fill\"\n");
    space.write("a.typ", "#let x = 0");
    let stdin = "lorem  ipsum   dolor sit amet, consectetur   adipiscing elit.";

    // An unrelated flag must not reset the configured width and wrapping.
    typstyle_cmd_snapshot!(space.cli().args(["--check"]).pass_stdin(stdin), @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    ");

    // `--wrap-text=none` is explicit and does override the file.
    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "--wrap-text=none"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let x = 0

    ----- stderr -----
    ");
}

#[test]
fn test_explicit_config_applies_to_every_input() {
    let space = Workspace::new();
    space.write("config/typstyle.toml", "indent-width = 4\n");
    space.write("nested/typstyle.toml", "indent-width = 8\n");
    space.write("nested/a.typ", "#let f(x) = {\nx\n}");

    // `--config` bypasses discovery for every input, including nested ones.
    typstyle_cmd_snapshot!(space.cli().args(["--config", "config/typstyle.toml", "nested/a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
        x
    }

    ----- stderr -----
    ");
}

#[test]
fn test_explicit_config_missing_file() {
    let space = Workspace::new();
    space.write("a.typ", "#let a  =  0");

    typstyle_cmd_snapshot!(space.cli().args(["--config", "nope.toml", "a.typ"]), @r"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
      Cause: failed to read [TEMP_PATH]/project/nope.toml
      Cause: No such file or directory (os error 2)
    ");
}

#[test]
fn test_isolated_ignores_config_file() {
    let space = Workspace::new();
    space.write("typstyle.toml", "indent-width = 8\n");
    space.write("a.typ", "#let f(x) = {\nx\n}");

    typstyle_cmd_snapshot!(space.cli().args(["--isolated", "a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
      x
    }

    ----- stderr -----
    ");
}

#[test]
fn test_stdin_filename_selects_config() {
    let space = Workspace::new();
    space.write("other/typstyle.toml", "indent-width = 4\n");
    let stdin = "#let f(x) = {\nx\n}";

    typstyle_cmd_snapshot!(space.cli().args(["--stdin-filename", "other/doc.typ"]).pass_stdin(stdin), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
        x
    }

    ----- stderr -----
    ");
}

#[test]
fn test_invalid_config_file_is_rejected() {
    let mut space = Workspace::new();
    space.write("typstyle.toml", "line-width = \"wide\"\n");
    space.write_tracked("a.typ", "#let a  =  0");

    let output = space.cli().args(["a.typ"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse"), "{stderr}");
    assert!(stderr.contains("typstyle.toml"), "{stderr}");
    assert!(
        stderr.contains("invalid type: string \"wide\", expected usize"),
        "{stderr}"
    );

    assert!(space.all_unmodified());
}

#[test]
fn test_unknown_config_key_is_rejected() {
    let mut space = Workspace::new();
    space.write("typstyle.toml", "indent-wdith = 4\n");
    space.write_tracked("a.typ", "#let a  =  0");

    let output = space.cli().args(["a.typ"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unknown field `indent-wdith`"), "{stderr}");
    assert!(stderr.contains("`indent-width`"), "{stderr}");
    assert!(!stderr.contains("a.typ"), "{stderr}");

    assert!(space.all_unmodified());
}

#[test]
fn test_invalid_config_values_are_rejected() {
    for (name, content) in [
        ("negative", "line-width = -1\n"),
        ("float", "line-width = 100.5\n"),
        ("enumerated", "line-ending = \"preserve-ish\"\n"),
        ("boolean-as-number", "collapse-markup-spaces = 1\n"),
    ] {
        let mut space = Workspace::new();
        space.write("typstyle.toml", content);
        space.write_tracked("a.typ", "#let a  =  0");

        let output = space.cli().args(["a.typ"]).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "case {name}");
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("failed to parse"),
            "case {name}"
        );
        assert!(space.all_unmodified(), "case {name}");
    }
}

#[test]
fn test_invalid_config_value_in_enumerated_option_is_rejected() {
    let mut space = Workspace::new();
    space.write("typstyle.toml", "wrap-mode = \"none-ish\"\n");
    space.write_tracked("a.typ", "#let a  =  0");

    let output = space.cli().args(["a.typ"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse"), "{stderr}");
    assert!(stderr.contains("wrap-mode"), "{stderr}");

    assert!(space.all_unmodified());
}

#[test]
fn test_config_file_selects_line_ending() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-ending = \"crlf\"\n");
    space.write("a.typ", "#let x  =  1\n");

    let output = space.cli().arg("a.typ").output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"#let x = 1\r\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn test_config_file_applies_to_inplace_and_diff() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-width = 20\nwrap-mode = \"fill\"\n");
    space.write(
        "a.typ",
        "lorem  ipsum   dolor sit amet, consectetur   adipiscing elit.\n",
    );

    // The same configuration applies in diff mode...
    let output = space.cli().args(["--diff", "a.typ"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let diff = String::from_utf8(output.stdout).unwrap();
    assert!(diff.contains("-lorem  ipsum   dolor sit amet, consectetur   adipiscing elit."));
    assert!(diff.contains("+consectetur"));

    // ...and in in-place mode.
    let output = space.cli().args(["-i", "a.typ"]).output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        space.read_string("a.typ"),
        "lorem ipsum dolor\nsit amet,\nconsectetur\nadipiscing elit.\n"
    );
}

#[test]
fn test_invalid_config_fails_before_writing() {
    let mut space = Workspace::new();
    space.write("typstyle.toml", "wrap-mode = \"nope\"\n");
    space.write_tracked("a.typ", "#let a  =  0");

    let output = space.cli().args(["-i", "a.typ"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("failed to parse")
    );

    // The input must not be modified when its configuration is invalid.
    assert_eq!(space.read_string("a.typ"), "#let a  =  0");
    assert!(space.all_unmodified());
}

#[test]
fn test_config_file_help_lists_selection_options() {
    let output = Workspace::new().cli().arg("-h").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(help.contains("--config <PATH>"));
    assert!(help.contains("--isolated"));
    assert!(help.contains("--stdin-filename <PATH>"));
}

#[test]
fn test_config_file_supports_every_formatter_option() {
    let space = Workspace::new();
    space.write(
        "typstyle.toml",
        r#"
line-width = 100
indent-width = 4
blank-lines-upper-bound = 0
reorder-import-items = false
wrap-mode = "fill"
"#,
    );
    space.write(
        "a.typ",
        "#import \"m.typ\": zzz, aaa\n\n#let f(x) = {\n\n\n x\n}",
    );

    typstyle_cmd_snapshot!(space.cli().args(["a.typ"]), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "m.typ": zzz, aaa

    #let f(x) = {
        x
    }

    ----- stderr -----
    "#);
}

#[test]
fn test_config_file_collapses_markup_spaces() {
    let space = Workspace::new();
    space.write(
        "typstyle.toml",
        "collapse-markup-spaces = true\nline-width = 100\n",
    );
    space.write("a.typ", "hello    world  with   spaces\n");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    hello world with spaces

    ----- stderr -----
    ");
}

#[test]
fn test_one_run_uses_the_config_of_each_directory() {
    let space = Workspace::new();
    space.write("typstyle.toml", "indent-width = 4\n");
    space.write("examples/typstyle.toml", "indent-width = 8\n");
    space.write("a.typ", "#let f(x) = {\nx\n}");
    space.write("examples/b.typ", "#let g(y) = {\ny\n}");

    typstyle_cmd_snapshot!(space.cli().args(["a.typ", "examples/b.typ"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
        x
    }
    #let g(y) = {
            y
    }

    ----- stderr -----
    ");
}

#[test]
fn test_directory_input_resolves_config_per_file() {
    let space = Workspace::new();
    space.write("typstyle.toml", "indent-width = 4\n");
    space.write("src/typstyle.toml", "indent-width = 3\n");
    space.write("src/a.typ", "#let f(x) = {\nx\n}");
    space.write("src/deep/b.typ", "#let g(y) = {\ny\n}");

    typstyle_cmd_snapshot!(space.cli().args(["src"]), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    #let f(x) = {
       x
    }
    #let g(y) = {
       y
    }

    ----- stderr -----
    ");
}

#[test]
fn test_explicit_boolean_overrides_import_sorting_both_ways() {
    let space = Workspace::new();
    let stdin = r#"#import "m.typ": zzz, aaa"#;

    // The file disables sorting; `--no-reorder-import-items=false` must
    // re-enable it rather than leaving the file's value in place.
    space.write("typstyle.toml", "reorder-import-items = false\n");
    typstyle_cmd_snapshot!(space.cli().args(["--no-reorder-import-items=false"]).pass_stdin(stdin), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "m.typ": aaa, zzz

    ----- stderr -----
    "#);

    // The file enables sorting; `--no-reorder-import-items` must disable it.
    space.write("typstyle.toml", "reorder-import-items = true\n");
    typstyle_cmd_snapshot!(space.cli().args(["--no-reorder-import-items"]).pass_stdin(stdin), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    #import "m.typ": zzz, aaa

    ----- stderr -----
    "#);
}

#[test]
fn test_empty_directory_still_validates_explicit_config() {
    let space = Workspace::new();
    space.write("typstyle.toml", "line-width = \"wide\"\n");
    std::fs::create_dir_all(space.project_path().join("empty")).unwrap();

    // A malformed discovered file is not read when there is nothing to format,
    // but an unusable explicitly selected file must still be reported.
    let output = space
        .cli()
        .args(["--config", "typstyle.toml", "empty"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to parse"), "{stderr}");

    // A missing explicitly selected file is reported as well.
    let output = space
        .cli()
        .args(["--config", "missing.toml", "empty"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("failed to read"), "{stderr}");

    // Without an explicit file, an empty directory stays a successful no-op.
    let output = space.cli().arg("empty").output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("No Typst files found")
    );
}
