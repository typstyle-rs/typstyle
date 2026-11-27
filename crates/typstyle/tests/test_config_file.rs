//! Integration tests for layered TOML configuration.

mod common;

use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

use common::Workspace;

fn run_command(mut command: Command, stdin: &str) -> Output {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn run(space: &Workspace, args: &[&str], stdin: &str) -> Output {
    let mut command = space.cli();
    command.args(args);
    run_command(command, stdin)
}

#[test]
fn project_config_applies_and_cli_boolean_overrides_it() {
    let space = Workspace::new();
    space.write("typstyle.toml", "reorder_import_items = false\n");
    let stdin = r#"#import "module.typ": xyz, func as renamed, a.b.c"#;

    let from_file = run(&space, &[], stdin);
    assert!(from_file.status.success());
    assert_eq!(
        String::from_utf8(from_file.stdout).unwrap(),
        "#import \"module.typ\": xyz, func as renamed, a.b.c\n"
    );

    let from_cli = run(&space, &["--reorder-import-items=true"], stdin);
    assert!(from_cli.status.success());
    assert_eq!(
        String::from_utf8(from_cli.stdout).unwrap(),
        "#import \"module.typ\": a.b.c, func as renamed, xyz\n"
    );
}

#[test]
fn explicit_config_path_replaces_project_discovery() {
    let space = Workspace::new();
    space.write("typstyle.toml", "reorder_import_items = true\n");
    space.write("nested/other.toml", "reorder_import_items = false\n");
    let stdin = r#"#import "module.typ": xyz, a.b.c"#;

    let output = run(&space, &["--config", "nested/other.toml"], stdin);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "#import \"module.typ\": xyz, a.b.c\n"
    );
}

#[test]
fn user_config_is_lower_priority_than_project_and_cli() {
    let space = Workspace::new();
    space.write("user.toml", "line_ending = \"crlf\"\n");
    space.write("typstyle.toml", "line_ending = \"lf\"\n");
    let stdin = "#let a  =  1";

    let project = run(&space, &[], stdin);
    assert!(project.status.success());
    assert_eq!(String::from_utf8(project.stdout).unwrap(), "#let a = 1\n");

    let mut command = space.cli();
    command
        .env("TYPSTYLE_CONFIG", space.project_path().join("user.toml"))
        .args(["--line-ending=crlf"]);
    let cli = run_command(command, stdin);
    assert!(cli.status.success());
    assert_eq!(String::from_utf8(cli.stdout).unwrap(), "#let a = 1\r\n");

    let mut command = space.cli();
    command.env("TYPSTYLE_CONFIG", space.project_path().join("user.toml"));
    let user = run_command(command, stdin);
    assert!(user.status.success());
    assert_eq!(
        String::from_utf8(user.stdout).unwrap(),
        "#let a = 1\n",
        "the project configuration must override the user configuration"
    );
}

#[test]
fn unknown_configuration_keys_are_rejected() {
    let space = Workspace::new();
    space.write("typstyle.toml", "unknown_option = 1\n");

    let output = run(&space, &[], "");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout).unwrap().is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("unknown field `unknown_option`"),
        "{stderr}"
    );
}
