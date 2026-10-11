//! Tests for partial formatting of a source range.

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

fn parses(text: &str) -> bool {
    !Source::detached(text).root().diagnosis().errors
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

#[test]
fn markup_fragment_preserves_comment_boundaries() {
    let input = "// hello\ntext\n";
    let output = apply_partial_format(input, 0..1);
    assert_eq!(output, input);
}

#[test]
fn markup_fragment_does_not_add_document_newline() {
    for (input, range, expected) in [
        ("#let x=1\n#let y=2\n", 0..1, "#"),
        (
            "before\n```text\na  \nb\n```\nafter\n",
            3..9,
            "before\n```text\na  \nb\n```",
        ),
    ] {
        let edit = format_range(input, range);
        assert_eq!(edit.content, expected);
        let output = apply_edit(input, edit);
        assert!(parses(&output));
    }
}

#[test]
fn range_ir_preserves_fragment_layout() {
    let input = "before\n```text\na\n```\nafter\n";
    let text = format_range(input, 0..20);
    let ir = format_range_ir(input, 0..20);
    assert_eq!(ir.source_range, text.source_range);
    insta::assert_snapshot!(ir.content, @r#"
    Group([
      "before",
      HardLine,
      "```",
      "text",
      HardLine,
      "a",
      HardLine,
      "```",
    ])
    "#);
}

#[test]
fn partial_edit_preserves_semicolon_separator() {
    let formatter = Typstyle::default();
    for (input, selected) in [
        ("#let x={let a=1; a}\n", "let a=1;"),
        ("#let x={let a=1 /* comment */; a}\n", "/* comment */;"),
        (
            "#let x={let a=1; /* comment */ a}\n",
            "let a=1; /* comment */",
        ),
        ("#let x={let a=1; // comment\n a}\n", "let a=1; // comment"),
        (
            "#let x={let a=1;\n /* comment */\n a}\n",
            "let a=1;\n /* comment */",
        ),
        (
            "#let x={let a=1; /* comment\n continued */ a}\n",
            "let a=1; /* comment\n continued */",
        ),
        ("#let x={let a=1; a}\n", ";"),
    ] {
        let output = apply_partial_format(input, find_range(input, selected));
        assert!(
            parses(&output),
            "partial edit produced invalid syntax: {output:?}"
        );
        assert_eq!(
            formatter.format_text(&output).render().unwrap(),
            formatter.format_text(input).render().unwrap(),
            "input={input:?}, selected={selected:?}"
        );
    }
}

#[test]
fn code_range_edits_preserve_valid_syntax_at_every_boundary() {
    for input in [
        "#let x={let a=1; let b=2; b}\n",
        "#let x={let a=1 /* comment */; a}\n",
        "#let x={let a=1; /* comment */ a}\n",
        "#let x={let a=1;\n /* comment */\n a}\n",
        "#let x={let a=1; // comment\n a}\n",
        "#let x={let 名称=1; 名称}\n",
    ] {
        let boundaries: Vec<_> = input
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once(input.len()))
            .collect();
        for (index, &start) in boundaries.iter().enumerate() {
            for &end in &boundaries[index..] {
                let edit = format_range(input, start..end);
                let output = apply_edit(input, edit);
                assert!(
                    parses(&output),
                    "input={input:?}, range={start}..{end}, output={output:?}"
                );
            }
        }
    }
}

#[test]
fn code_boundary_keeps_whitespace_around_semicolon() {
    // Non-expression trivia adjacent to the separator is boundary trivia and
    // is rendered verbatim so the applied edit cannot shift it.
    let input = "#let x={let a=1 ;  a}\n";
    let edit = format_range(input, find_range(input, "let a=1 ;"));
    assert_eq!(edit.content, "let a = 1 ;");
    assert!(parses(&apply_edit(input, edit)));

    let edit = format_range(input, find_range(input, ";  a"));
    assert_eq!(edit.content, ";  a");
    assert!(parses(&apply_edit(input, edit)));
}
