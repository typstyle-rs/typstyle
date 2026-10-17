//! Integration tests for ignore-aware file discovery: standard ignore files,
//! the opt-out switch, explicit inputs, hidden entries, and failure handling.

mod common;

use std::{path::Path, process::Command};

use common::Workspace;

/// An unformatted Typst source, so that selection is observable in every mode.
const UNFORMATTED: &str = "#let a  =  0";

/// Runs `typstyle --check` with the given selection arguments and returns the
/// sorted, project-relative paths that were selected for formatting.
fn selected(space: &Workspace, args: &[&str]) -> Vec<String> {
    let mut paths = selected_in(space, space.project_path(), args);
    paths.sort();
    paths
}

/// Like [`selected`], but runs the CLI from an arbitrary directory.
///
/// Reported paths keep the platform's native separators; they are normalized
/// to `/` so that assertions can compare the same relative paths everywhere.
fn selected_in(space: &Workspace, dir: &Path, args: &[&str]) -> Vec<String> {
    let out = space
        .cli()
        .current_dir(dir)
        .arg("--check")
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("Would reformat: "))
        .map(|path| path.replace('\\', "/"))
        .collect()
}

/// Returns a command that runs the CLI in the project directory with the
/// given arguments.
fn cli(space: &Workspace, args: &[&str]) -> Command {
    let mut cmd = space.cli();
    cmd.args(args);
    cmd
}

#[test]
fn test_ignore_file_respected_by_default() {
    let space = Workspace::new();
    space.write(".ignore", "ignored.typ\n");
    space.write("ignored.typ", UNFORMATTED);
    space.write("included.typ", UNFORMATTED);

    assert_eq!(selected(&space, &["."]), ["included.typ"]);
    assert_eq!(
        selected(&space, &[".", "--respect-gitignore"]),
        ["included.typ"]
    );
    assert_eq!(
        selected(&space, &[".", "--no-respect-gitignore"]),
        ["ignored.typ", "included.typ"]
    );
}

#[test]
fn test_gitignore_requires_repository() {
    let space = Workspace::new();
    space.write(".gitignore", "ignored.typ\n");
    space.write("ignored.typ", UNFORMATTED);

    // Outside a repository, Git rules do not apply.
    assert_eq!(selected(&space, &["."]), ["ignored.typ"]);

    space.git_init();
    assert!(selected(&space, &["."]).is_empty());
    assert_eq!(
        selected(&space, &[".", "--no-respect-gitignore"]),
        ["ignored.typ"]
    );
}

#[test]
fn test_git_local_excludes() {
    let space = Workspace::new();
    space.git_init();
    space.write(".git/info/exclude", "excluded.typ\n");
    space.write("excluded.typ", UNFORMATTED);
    space.write("included.typ", UNFORMATTED);

    assert_eq!(selected(&space, &["."]), ["included.typ"]);
    assert_eq!(
        selected(&space, &[".", "--no-respect-gitignore"]),
        ["excluded.typ", "included.typ"]
    );
}

#[test]
fn test_global_gitignore() {
    let space = Workspace::new();
    space.write(
        "gitconfig-global",
        "[core]\n\texcludesFile = global-ignore\n",
    );
    space.write("global-ignore", "global.typ\n");
    space.write("global.typ", UNFORMATTED);

    // The global rules also require a repository.
    assert_eq!(selected(&space, &["."]), ["global.typ"]);

    space.git_init();
    assert!(selected(&space, &["."]).is_empty());
    assert_eq!(
        selected(&space, &[".", "--no-respect-gitignore"]),
        ["global.typ"]
    );
}

#[test]
fn test_parent_rules_from_subdirectory() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "*.typ\n!keep.typ\n");
    space.write("sub/keep.typ", UNFORMATTED);
    space.write("sub/drop.typ", UNFORMATTED);
    space.write("sub/generated/also-drop.typ", UNFORMATTED);
    space.write("sub/.gitignore", "generated/\n");

    let sub = space.project_path().join("sub");
    let mut from_sub = selected_in(&space, &sub, &["."]);
    from_sub.sort();
    assert_eq!(from_sub, ["keep.typ"]);
    let mut opted_out = selected_in(&space, &sub, &[".", "--no-respect-gitignore"]);
    opted_out.sort();
    assert_eq!(
        opted_out,
        ["drop.typ", "generated/also-drop.typ", "keep.typ"]
    );
}

#[test]
fn test_ignored_directory_pruning_and_nested_negation() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "generated/\n*.typ\n!keep.typ\n");
    space.write("keep.typ", UNFORMATTED);
    space.write("drop.typ", UNFORMATTED);
    // A nested rule cannot re-include entries below a pruned directory.
    space.write("generated/.gitignore", "!keep.typ\n");
    space.write("generated/keep.typ", UNFORMATTED);

    assert_eq!(selected(&space, &["."]), ["keep.typ"]);
    // An explicit root overrides the exclusion of the directory itself, but
    // patterns matching individual descendants still apply.
    assert_eq!(selected(&space, &["generated"]), ["generated/keep.typ"]);
}

#[test]
fn test_explicit_files_override_ignores() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "ignored.typ\n.hidden.typ\n");
    space.write("ignored.typ", UNFORMATTED);
    space.write(".hidden.typ", UNFORMATTED);
    space.write("note.txt", UNFORMATTED);

    assert_eq!(
        selected(&space, &["ignored.typ", ".hidden.typ", "note.txt"]),
        [".hidden.typ", "ignored.typ", "note.txt"]
    );

    let out = cli(&space, &["-i", "ignored.typ", ".hidden.typ", "note.txt"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(space.read_string("ignored.typ"), "#let a = 0\n");
    assert_eq!(space.read_string(".hidden.typ"), "#let a = 0\n");
    assert_eq!(space.read_string("note.txt"), "#let a = 0\n");
}

#[test]
fn test_hidden_entries_skipped_without_respect() {
    let space = Workspace::new();
    space.write(".ignore", "!.hidden.typ\n");
    space.write(".hidden.typ", UNFORMATTED);
    space.write(".hidden/inner.typ", UNFORMATTED);
    space.write("visible.typ", UNFORMATTED);

    for args in [
        vec!["."],
        vec![".", "--no-respect-gitignore"],
        vec![".", "--respect-gitignore"],
    ] {
        assert_eq!(selected(&space, &args), ["visible.typ"], "args: {args:?}");
    }
}

#[test]
fn test_explicit_hidden_and_ignored_directory_roots() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "generated/\n");
    space.write("generated/a.typ", UNFORMATTED);
    space.write("generated/.b.typ", UNFORMATTED);
    space.write(".hidden/c.typ", UNFORMATTED);

    // Hidden descendants stay skipped even for an explicit hidden root.
    assert_eq!(selected(&space, &[".hidden"]), [".hidden/c.typ"]);
    // An explicit root overrides the exclusion of the directory itself.
    assert_eq!(selected(&space, &["generated"]), ["generated/a.typ"]);
}

#[test]
fn test_overlapping_inputs_and_ordering() {
    let space = Workspace::new();
    space.write("a.typ", UNFORMATTED);
    space.write("b.typ", UNFORMATTED);
    space.write("sub/c.typ", UNFORMATTED);

    // The expanded directory sorts the union, and each file appears once.
    assert_eq!(
        selected(&space, &["b.typ", "."]),
        ["a.typ", "b.typ", "sub/c.typ"]
    );

    // Explicit-only inputs keep their argument order.
    assert_eq!(
        selected_in(&space, space.project_path(), &["b.typ", "a.typ"]),
        ["b.typ", "a.typ"]
    );
}

#[test]
fn test_extension_filtering() {
    let space = Workspace::new();
    space.write("lower.typ", UNFORMATTED);
    space.write("upper.TYP", UNFORMATTED);
    space.write("note.txt", UNFORMATTED);

    // Only `.typ` files are discovered, and the extension is matched exactly.
    assert_eq!(selected(&space, &["."]), ["lower.typ"]);
    // The opt-out disables ignore-file filtering only.
    assert_eq!(
        selected(&space, &[".", "--no-respect-gitignore"]),
        ["lower.typ"]
    );
}

#[test]
#[cfg(unix)]
fn test_symlinks() {
    let space = Workspace::new();
    space.write("sub/real.typ", UNFORMATTED);
    std::os::unix::fs::symlink("sub/real.typ", space.project_path().join("link.typ")).unwrap();
    std::os::unix::fs::symlink("sub", space.project_path().join("linkdir")).unwrap();

    // Links found during discovery are not followed, in either mode.
    assert_eq!(selected(&space, &["."]), ["sub/real.typ"]);
    assert_eq!(
        selected(&space, &[".", "--no-respect-gitignore"]),
        ["sub/real.typ"]
    );

    // Explicitly supplied links keep their direct-file handling.
    assert_eq!(selected(&space, &["link.typ"]), ["link.typ"]);
    assert_eq!(selected(&space, &["linkdir"]), ["linkdir/real.typ"]);

    let out = cli(&space, &["-i", "link.typ"]).output().unwrap();
    assert!(out.status.success());
    assert_eq!(space.read_string("sub/real.typ"), "#let a = 0\n");
}

#[test]
fn test_switch_forms_and_placement() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "ignored.typ\n");
    space.write("ignored.typ", UNFORMATTED);
    space.write("included.typ", UNFORMATTED);

    // The default and opt-out outcomes are covered by
    // `test_ignore_file_respected_by_default`; the last occurrence wins in
    // either order.
    assert_eq!(
        selected(
            &space,
            &[".", "--respect-gitignore", "--no-respect-gitignore"]
        ),
        ["ignored.typ", "included.typ"]
    );
    assert_eq!(
        selected(
            &space,
            &[".", "--no-respect-gitignore", "--respect-gitignore"]
        ),
        ["included.typ"]
    );

    // Switches may precede the positional inputs and do not consume them.
    assert_eq!(
        selected(&space, &["--no-respect-gitignore", "."]),
        ["ignored.typ", "included.typ"]
    );

    // A boolean value is rejected.
    let out = cli(&space, &["--respect-gitignore=true", "."])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("unexpected value"), "stderr: {stderr}");
}

#[test]
fn test_paths_after_double_dash() {
    let space = Workspace::new();
    space.write("--no-respect-gitignore", UNFORMATTED);

    // After `--`, the switch name is a path, not an argument.
    let out = cli(&space, &["-i", "--", "--no-respect-gitignore"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(space.read_string("--no-respect-gitignore"), "#let a = 0\n");
}

#[test]
fn test_same_selection_in_every_mode() {
    let space = Workspace::new();
    space.write(".ignore", "ignored.typ\n");
    space.write("ignored.typ", UNFORMATTED);
    space.write("included.typ", UNFORMATTED);

    let stdout_mode = cli(&space, &["."]).output().unwrap();
    assert!(stdout_mode.status.success());
    assert_eq!(
        String::from_utf8(stdout_mode.stdout).unwrap(),
        "#let a = 0\n"
    );

    let checked = cli(&space, &["--check", "."]).output().unwrap();
    assert_eq!(checked.status.code(), Some(1));
    let diffed = cli(&space, &["--diff", "."]).output().unwrap();
    assert_eq!(diffed.status.code(), Some(1));
    let diff = String::from_utf8(diffed.stdout).unwrap();
    assert!(diff.contains("included.typ"), "diff: {diff}");
    assert!(!diff.contains("ignored.typ"), "diff: {diff}");

    let inplace = cli(&space, &["-i", "."]).output().unwrap();
    assert!(inplace.status.success());
    assert_eq!(space.read_string("included.typ"), "#let a = 0\n");
    assert_eq!(space.read_string("ignored.typ"), UNFORMATTED);
}

#[test]
fn test_stdin_accepts_switches_without_discovery() {
    let space = Workspace::new();
    space.write(".ignore", "*.typ\n");
    space.write("ignored.typ", UNFORMATTED);

    for args in [vec!["--no-respect-gitignore"], vec!["--respect-gitignore"]] {
        let mut cmd = cli(&space, &args);
        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(UNFORMATTED.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "args: {args:?}");
        assert_eq!(String::from_utf8(out.stdout).unwrap(), "#let a = 0\n");
    }
    assert_eq!(space.read_string("ignored.typ"), UNFORMATTED);
}

#[test]
fn test_everything_ignored_warns_and_succeeds() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "*.typ\n");
    space.write("a.typ", UNFORMATTED);

    let out = cli(&space, &["-i", "-v", "."]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("No Typst files found under the given path(s)."),
        "stderr: {stderr}"
    );
    assert_eq!(space.read_string("a.typ"), UNFORMATTED);
}

#[test]
fn test_malformed_ignore_rule_fails_before_formatting() {
    let space = Workspace::new();
    // A dangling escape is rejected by the glob parser.
    space.write(".ignore", "a\\\n");
    space.write("a.typ", UNFORMATTED);

    let out = cli(&space, &["-i", "."]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(".ignore"), "stderr: {stderr}");
    assert_eq!(space.read_string("a.typ"), UNFORMATTED);
}

#[test]
#[cfg(unix)]
fn test_traversal_failure_fails_before_formatting() {
    use std::os::unix::fs::PermissionsExt;

    let space = Workspace::new();
    space.write("a.typ", UNFORMATTED);
    let locked = space.project_path().join("locked");
    std::fs::create_dir_all(&locked).unwrap();
    space.write("locked/b.typ", UNFORMATTED);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

    // Readers with CAP_DAC_OVERRIDE (e.g. root) cannot trigger the failure.
    let readable = std::fs::read_dir(&locked).is_ok();
    let out = cli(&space, &["-i", "."]).output().unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();

    if readable {
        return;
    }
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(space.read_string("a.typ"), UNFORMATTED);
}

#[test]
fn test_verbose_output_reports_only_typstyle_records() {
    let space = Workspace::new();
    space.git_init();
    space.write(".gitignore", "ignored.typ\n");
    space.write("ignored.typ", UNFORMATTED);
    space.write("included.typ", UNFORMATTED);

    let out = cli(&space, &["-i", "-v", "."]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // The traversal dependencies log ignore-file internals at debug level.
    assert!(
        !stdout.contains("opened gitignore file"),
        "stdout: {stdout}"
    );
    assert!(!stdout.contains("ignoring "), "stdout: {stdout}");
    assert!(
        stdout.contains("Successfully formatted 1 file"),
        "stdout: {stdout}"
    );
}

#[test]
fn test_help_lists_both_switches() {
    let space = Workspace::new();
    let out = cli(&space, &["--help"]).output().unwrap();
    assert!(out.status.success());
    let help = String::from_utf8(out.stdout).unwrap();
    let help = help.split_whitespace().collect::<Vec<_>>().join(" ");

    assert!(help.contains("File Selection:"), "help: {help}");
    assert!(help.contains("--respect-gitignore"), "help: {help}");
    assert!(help.contains("--no-respect-gitignore"), "help: {help}");
}
