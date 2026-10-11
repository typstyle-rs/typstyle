//! Generated line endings must not rewrite literal source or change layout.

use typst_syntax::Source;
use typstyle_core::{Config, LineEnding, Typstyle, WrapMode};

#[test]
fn crlf_preserves_literal_newlines_and_partial_layout() {
    let input = "#let x  =  \"a\nb\"\n#let y = /* @typstyle off */ (1,\n 2)\n#let z = `a\nb`\n";
    let expected =
        "#let x = \"a\nb\"\r\n#let y = /* @typstyle off */ (1,\n 2)\r\n#let z = `a\nb`\r\n";
    let typstyle = Typstyle::new(Config::new().with_line_ending(LineEnding::Crlf));
    for input in ["#let x  =  1", "#let x  =  1\n", "#let x  =  1\r\n"] {
        assert_eq!(
            typstyle.format_text(input).render().unwrap(),
            "#let x = 1\r\n"
        );
    }
    assert_eq!(typstyle.format_text(input).render().unwrap(), expected);
    assert_eq!(typstyle.format_text(expected).render().unwrap(), expected);
    let partial = typstyle
        .format_source_range(Source::detached(input), 0..input.len())
        .unwrap();
    assert_eq!(partial.content, expected);
    let fragment = "#let x = {\n  let a = 1\n\n  a\n}\n";
    let start = fragment.find('{').unwrap();
    let end = fragment.rfind('}').unwrap() + 1;
    let partial = typstyle
        .format_source_range(Source::detached(fragment), start..end)
        .unwrap();
    assert_eq!(partial.source_range, start..end);
    assert_eq!(partial.content, fragment[start..end].replace('\n', "\r\n"));
}

#[test]
fn explicit_lf_keeps_crlf_in_verbatim_raw_text() {
    let input = "#let x  =  `a\r\nb`\r\n";
    let output = Typstyle::new(Config::new().with_line_ending(LineEnding::Lf))
        .format_text(input)
        .render()
        .unwrap();
    assert_eq!(output, "#let x = `a\r\nb`\n");
}

#[test]
fn default_preservation_resolves_each_document_independently() {
    let formatter = Typstyle::default();
    for (input, expected) in [
        ("#let x  =  1\r\n", "#let x = 1\r\n"),
        ("#let x  =  1\n", "#let x = 1\n"),
        ("#let x  =  1", "#let x = 1\n"),
        (
            "#let x  =  1\r\n#let y  =  2",
            "#let x = 1\r\n#let y = 2\r\n",
        ),
        (
            "#let x  =  \"a\r\nb\"\r\n#let y  =  ```a\r\nb```\r\n",
            "#let x = \"a\r\nb\"\r\n#let y = ```a\r\nb```\r\n",
        ),
        (
            "#let x = /* @typstyle off */ (1,\r\n 2)\r\n#let y  =  \"a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}\"\r\n",
            "#let x = /* @typstyle off */ (1,\r\n 2)\r\n#let y = \"a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}\"\r\n",
        ),
    ] {
        assert_eq!(formatter.format_text(input).render().unwrap(), expected);
        assert_eq!(formatter.format_text(expected).render().unwrap(), expected);
    }
}

#[test]
fn preserve_fallbacks_match_lf_output() {
    let lf = Typstyle::new(Config::new().with_line_ending(LineEnding::Lf));
    let preserved = Typstyle::default();
    for input in [
        "#let x  =  1\n",
        "#let x  =  1\r",
        "#let x  =  1",
        "#let x  =  1\r\n#let y  =  2\n",
        "#let x  =  1\n#let y  =  2\r\n",
        "#let x  =  1\r\n#let y  =  2\r",
        "#let x  =  \"a\nb\"\r\n",
        "#let x  =  \"a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}\"\n",
    ] {
        let expected = lf.format_text(input).render().unwrap();
        let actual = preserved.format_text(input).render().unwrap();
        assert_eq!(actual, expected, "input={input:?}");
        assert_eq!(preserved.format_text(&actual).render().unwrap(), actual);
    }
}

#[test]
fn crlf_preserves_wrapping_and_deferred_indentation() {
    let inputs = [
        "#let outer = {\n  let a = 1\n\n  a\n}\n",
        "#foo(1, 2, 3, 4) // comment\n",
        "One sentence. Another longer sentence.\n",
        "/* comment\n\nmore comment */\n",
    ];
    for mode in [
        WrapMode::None,
        WrapMode::Fill,
        WrapMode::Sentence,
        WrapMode::FillSentence,
    ] {
        for width in [0, 20, 80] {
            let config = Config::new().with_width(width).with_wrap_mode(mode);
            let lf = Typstyle::new(config.clone().with_line_ending(LineEnding::Lf));
            let crlf = Typstyle::new(config.with_line_ending(LineEnding::Crlf));
            for input in inputs {
                let expected = lf
                    .format_text(input)
                    .render()
                    .unwrap()
                    .replace('\n', "\r\n");
                let output = crlf.format_text(input).render().unwrap();
                assert_eq!(
                    output, expected,
                    "input={input:?}, mode={mode:?}, width={width}"
                );
                assert!(output.lines().all(|line| !line.ends_with([' ', '\t'])));
                assert_eq!(crlf.format_text(&output).render().unwrap(), output);
            }
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn serialized_line_endings_keep_existing_configurations_compatible() {
    let default: Config = serde_json::from_str("{}").unwrap();
    assert_eq!(default.line_ending, LineEnding::Preserve);
    assert_eq!(LineEnding::default(), LineEnding::Preserve);
    for (ending, name) in [
        (LineEnding::Lf, "lf"),
        (LineEnding::Crlf, "crlf"),
        (LineEnding::Preserve, "preserve"),
    ] {
        let config = Config::new().with_line_ending(ending);
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["line_ending"], name);
        assert_eq!(serde_json::from_value::<Config>(value).unwrap(), config);
    }
}

#[test]
fn range_preservation_inspects_the_whole_original_source() {
    let cases = [
        (
            "#let before = 0\r\n#let x = foo(1, 2, 3)\r\n",
            LineEnding::Crlf,
        ),
        ("#let before = 0\n#let x = foo(1, 2, 3)\r\n", LineEnding::Lf),
        ("#let before = 0\r#let x = foo(1, 2, 3)\r\n", LineEnding::Lf),
        ("#let x = foo(1, 2, 3)", LineEnding::Lf),
        (
            "#let before = \"a\nb\"\r\n#let x = foo(1, 2, 3)\r\n",
            LineEnding::Lf,
        ),
    ];
    for (input, resolved) in cases {
        let start = input.find("foo").unwrap();
        let end = input.rfind(')').unwrap() + 1;
        let source = Source::detached(input);
        let config = Config::new().with_width(10);
        let expected = Typstyle::new(config.clone().with_line_ending(resolved))
            .format_source_range(source.clone(), start..end)
            .unwrap();
        let preserved = Typstyle::new(config.with_line_ending(LineEnding::Preserve));
        let actual = preserved
            .format_source_range(source.clone(), start..end)
            .unwrap();
        assert_eq!(actual.source_range, expected.source_range);
        assert_eq!(actual.content, expected.content, "input={input:?}");
        assert!(actual.content.contains('\n'));
        assert_eq!(
            preserved
                .format_source_range(source.clone(), 0..input.len())
                .unwrap()
                .content,
            preserved.format_source(source).render().unwrap(),
        );
        let mut edited = input.to_owned();
        edited.replace_range(actual.source_range.clone(), &actual.content);
        assert_eq!(
            &edited[..actual.source_range.start],
            &input[..actual.source_range.start]
        );
        assert!(edited.ends_with(&input[actual.source_range.end..]));
    }
}
