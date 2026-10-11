//! CLI line-ending integration tests: arguments, stdout, check, diff, and file writes.

mod common;

use common::Workspace;
use insta_cmd::{Spawn, SpawnExt};

#[test]
fn test_line_ending_args_select_core_policy() {
    let space = Workspace::new();
    for (args, forced_ending) in [
        (&[][..], None),
        (&["--line-ending=lf"][..], Some("\n")),
        (&["--line-ending", "crlf"][..], Some("\r\n")),
        (&["--line-ending=preserve"][..], None),
    ] {
        for source_ending in ["\n", "\r\n"] {
            space.write("input.typ", format!("#let x  =  1{source_ending}"));
            let output = space.cli().args(args).arg("input.typ").output().unwrap();
            assert!(output.status.success());
            assert_eq!(
                output.stdout,
                format!("#let x = 1{}", forced_ending.unwrap_or(source_ending)).as_bytes(),
                "args={args:?}, source_ending={source_ending:?}"
            );
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn test_line_ending_args_reject_invalid_or_missing_values() {
    let space = Workspace::new();
    for args in [
        &["--line-ending=auto"][..],
        &["--line-ending=crlf-preserve"][..],
        &["--line-ending"][..],
    ] {
        let output = space.cli().args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "args={args:?}");
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("--line-ending")
        );
    }
}

#[test]
fn test_line_ending_help_lists_values_and_default() {
    let output = Workspace::new().cli().arg("-h").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(help.contains("--line-ending <LINE_ENDING>"));
    assert!(help.contains("[default: preserve]"));
    assert!(help.contains("[possible values: lf, crlf, preserve]"));
}

#[test]
fn test_check_and_diff_respect_line_ending_policy_without_writing() {
    for (args, forced_ending) in [
        (&[][..], None),
        (&["--line-ending=lf"][..], Some("\n")),
        (&["--line-ending=crlf"][..], Some("\r\n")),
        (&["--line-ending=preserve"][..], None),
    ] {
        for input in ["#let a = 0\n", "#let a = 0\r\n", "#let a  =  0\r\n"] {
            let source_ending = if input.ends_with("\r\n") {
                "\r\n"
            } else {
                "\n"
            };
            let expected = format!("#let a = 0{}", forced_ending.unwrap_or(source_ending));
            let changed = input != expected;
            for mode in ["--check", "--diff"] {
                let mut space = Workspace::new();
                space.write_tracked("a.typ", input);
                let output = space
                    .cli()
                    .args(args)
                    .args(["a.typ", mode])
                    .output()
                    .unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(i32::from(changed)),
                    "args={args:?}, mode={mode}, input={input:?}"
                );
                if changed && mode == "--check" {
                    assert_eq!(output.stdout, b"Would reformat: a.typ\n");
                } else if changed {
                    assert!(output.stdout.starts_with(b"--- a.typ\n+++ a.typ\n"));
                } else {
                    assert!(output.stdout.is_empty());
                }
                assert!(output.stderr.is_empty());
                assert_eq!(space.read_string("a.typ"), input);
                assert!(space.all_unmodified());
            }
        }
    }
}

#[test]
fn test_inplace_applies_line_ending_policy_to_each_file() {
    for (args, forced_ending) in [
        (&[][..], None),
        (&["--line-ending=lf"][..], Some("\n")),
        (&["--line-ending=crlf"][..], Some("\r\n")),
        (&["--line-ending=preserve"][..], None),
    ] {
        let mut space = Workspace::new();
        space.write_tracked("lf.typ", "#let a  =  0\n");
        space.write_tracked("crlf.typ", "#let b  =  1\r\n");
        space.write_tracked(
            "unchanged.typ",
            format!("#let c = 2{}", forced_ending.unwrap_or("\r\n")),
        );
        let paths = ["lf.typ", "crlf.typ", "unchanged.typ"];
        let output = space
            .cli()
            .args(args)
            .args(paths)
            .arg("--inplace")
            .output()
            .unwrap();
        assert!(output.status.success(), "args={args:?}");
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert_eq!(
            space.read_string("lf.typ"),
            format!("#let a = 0{}", forced_ending.unwrap_or("\n"))
        );
        assert_eq!(
            space.read_string("crlf.typ"),
            format!("#let b = 1{}", forced_ending.unwrap_or("\r\n"))
        );
        assert!(space.is_unmodified("unchanged.typ"));
        let check = space
            .cli()
            .args(args)
            .args(paths)
            .arg("--check")
            .output()
            .unwrap();
        assert!(check.status.success(), "args={args:?}");
        assert!(check.stdout.is_empty());
        assert!(check.stderr.is_empty());
    }
}

#[test]
fn test_stdin_output_and_check_use_line_ending_policy() {
    let space = Workspace::new();
    for (args, forced_ending) in [
        (&[][..], None),
        (&["--line-ending=lf"][..], Some("\n")),
        (&["--line-ending=crlf"][..], Some("\r\n")),
        (&["--line-ending=preserve"][..], None),
    ] {
        for source_ending in ["\n", "\r\n"] {
            let input = format!("#let x = 1{source_ending}");
            let expected = format!("#let x = 1{}", forced_ending.unwrap_or(source_ending));
            let (_, output) = space
                .cli()
                .args(args)
                .pass_stdin(input.as_bytes())
                .spawn_with_info(None);
            assert!(output.status.success());
            assert_eq!(
                output.stdout,
                expected.as_bytes(),
                "args={args:?}, input={input:?}"
            );
            assert!(output.stderr.is_empty());
            for content in [&input, &expected] {
                let (_, check) = space
                    .cli()
                    .args(args)
                    .arg("--check")
                    .pass_stdin(content.as_bytes())
                    .spawn_with_info(None);
                assert_eq!(
                    check.status.code(),
                    Some(i32::from(content != &expected)),
                    "args={args:?}, input={content:?}"
                );
                assert!(check.stdout.is_empty());
                assert!(check.stderr.is_empty());
            }
        }
    }
}

#[test]
fn test_stdin_diff_keeps_control_lines_lf() {
    let space = Workspace::new();
    for (args, input, expected) in [
        (
            &[][..],
            &b"#let x  =  1\r\n"[..],
            &b"@@ -1 +1 @@\n-#let x  =  1\r\n+#let x = 1\r\n\n"[..],
        ),
        (
            &["--line-ending=crlf"][..],
            &b"#let x = 1\n"[..],
            &b"@@ -1 +1 @@\n-#let x = 1\n+#let x = 1\r\n\n"[..],
        ),
    ] {
        let (_, output) = space
            .cli()
            .args(args)
            .arg("--diff")
            .pass_stdin(input)
            .spawn_with_info(None);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn test_erroneous_stdin_bypasses_line_ending_policy() {
    let space = Workspace::new();
    let input = b"#let x = (\r\n";
    for policy in ["lf", "crlf", "preserve"] {
        let (_, output) = space
            .cli()
            .arg(format!("--line-ending={policy}"))
            .pass_stdin(input)
            .spawn_with_info(None);
        assert!(output.status.success());
        assert_eq!(output.stdout, input);
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("Failed to parse stdin")
        );
    }
}
