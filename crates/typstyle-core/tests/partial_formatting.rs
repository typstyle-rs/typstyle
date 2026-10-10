use std::ops::Range;

use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode, partial::RangeResult};

#[test]
fn full_range_uses_document_layout() {
    let cases = [
        "",
        " ",
        "\t\n \n",
        "text",
        "#let x = 1",
        "  text  \n\n\n",
        "\n\n#let x=1\n\n\n",
        "#let outer = {\n  let inner = {\n    let a = 1\n\n    a\n  }\n  inner\n}\n",
        "// comment  \n#let x=1\n",
        "你好 café 🌍\n",
        "#let x = \"a \nb\"\n",
        "```text\na  \nb\n```\n",
        "#let x = /* @typstyle off */ (1, \n 2)\n",
        "$\n  a &= b \\\n  // comment\n  long &= c\n$\n",
    ];
    for mode in [
        WrapMode::None,
        WrapMode::Fill,
        WrapMode::Sentence,
        WrapMode::FillSentence,
    ] {
        for width in [0, 20, 80] {
            let formatter = Typstyle::new(Config::new().with_width(width).with_wrap_mode(mode));
            for input in cases {
                let source = Source::detached(input);
                let expected = formatter.format_source(source.clone()).render().unwrap();
                let expected_ir = formatter.format_source(source.clone()).render_ir().unwrap();
                let result = formatter
                    .format_source_range(source.clone(), 0..input.len())
                    .unwrap();
                let result_ir = formatter
                    .format_source_range_ir(source, 0..input.len())
                    .unwrap();
                assert_eq!(result.source_range, 0..input.len());
                assert_eq!(
                    result.content, expected,
                    "input={input:?}, mode={mode:?}, width={width}"
                );
                assert_eq!(result_ir.source_range, result.source_range);
                assert_eq!(result_ir.content, expected_ir);
            }
        }
    }
}

fn find_range(input: &str, selected: &str) -> Range<usize> {
    let start = input.find(selected).unwrap();
    start..start + selected.len()
}

fn format_range(input: &str, range: Range<usize>) -> RangeResult {
    Typstyle::default()
        .format_source_range(Source::detached(input), range)
        .unwrap()
}

fn format_range_ir(input: &str, range: Range<usize>) -> RangeResult {
    Typstyle::default()
        .format_source_range_ir(Source::detached(input), range)
        .unwrap()
}

fn apply_edit(input: &str, edit: RangeResult) -> String {
    let mut output = input.to_owned();
    output.replace_range(edit.source_range, &edit.content);
    output
}

fn apply_partial_format(input: &str, range: Range<usize>) -> String {
    apply_edit(input, format_range(input, range))
}

#[test]
fn disabled_ancestor_preserves_descendant_selection() {
    let input = "#let x=/* @typstyle off */ {\n let a=1\n}\n#let y=3\n";
    let output = apply_partial_format(input, find_range(input, "let a=1"));
    assert_eq!(output, input);
}

#[test]
fn disabled_range_survives_unrelated_syntax_error() {
    let input = "#let x=/* @typstyle off */ (1, \n 2)\n#let broken=(,)\n";
    let selected = find_range(input, "(1, \n 2)");
    let output = apply_partial_format(input, selected.clone());
    assert_eq!(output, input);
    let ir = format_range_ir(input, selected.clone());
    assert_eq!(ir.source_range, selected);
    insta::assert_snapshot!(ir.content, @r#"
    MultilineText(
      "(1, \n 2)",
    )
    "#);
}

#[test]
fn disabled_range_preserves_blank_lines_and_crlf() {
    // The selected node is rendered verbatim: blank lines, trailing spaces,
    // and the source line endings must all survive byte-for-byte. Editors
    // commonly send CRLF text, and the verbatim path must not normalize it.
    for selected in ["{\n  a = 1  \n\n  b  \n}", "(1,\r\n 2)"] {
        let input = format!("#let x = /* @typstyle off */ {selected}\n#let y=3\n");
        let range = find_range(&input, selected);
        let edit = format_range(&input, range.clone());
        assert_eq!(edit.source_range, range);
        assert_eq!(edit.content, selected, "input={input:?}");
        assert_eq!(apply_edit(&input, edit), input);
    }
}

#[test]
fn selection_inside_raw_preserves_trailing_spaces() {
    // A selection covering only raw content is emitted as-is; no trailing
    // whitespace may be trimmed and no document newline may be added.
    let input = "before\n```text\na  \nb\n```\nafter\n";
    let range = find_range(input, "a  ");
    let edit = format_range(input, range.clone());
    assert_eq!(edit.source_range, range);
    assert_eq!(edit.content, "a  ");
    assert_eq!(apply_edit(input, edit), input);
}

#[test]
fn whitespace_selection_does_not_delete_code_separator() {
    let input = "#{let a=1\n\n a}\n";
    let start = input.find('\n').unwrap();
    for range in [start..start + 1, start + 1..start + 1] {
        let edit = format_range(input, range.clone());
        let ir = format_range_ir(input, range);
        assert_eq!(ir.source_range, edit.source_range);
        assert_eq!(ir.content, "");
        assert_eq!(apply_edit(input, edit), input);
    }
}

#[test]
fn partial_edits_preserve_source_whitespace() {
    let cases = [
        ("\"a \nb\"", "\"a \nb\""),
        ("```text\na  \nb\n```", "```text\na  \nb\n```"),
        ("{\n  let a=1\n\n  a\n}", "{\n  let a = 1\n\n  a\n}"),
        ("(1, \n 2)", "(1, \n 2)"),
    ];
    for (selected, expected) in cases {
        let input = format!("#let x = /* @typstyle off */ {selected}\n#let y=2\n");
        // Only the last case is disabled; the others test the normal renderer.
        let input = if selected.starts_with('(') {
            input
        } else {
            input.replace("/* @typstyle off */ ", "")
        };
        let range = find_range(&input, selected);
        let result = format_range(&input, range.clone());
        assert_eq!(result.source_range, range);
        assert_eq!(result.content, expected);
    }
}
