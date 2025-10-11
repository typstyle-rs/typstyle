use typstyle_core::{Config, Typstyle, WrapMode};

#[test]
fn nested_blank_lines_preserve_breaks_without_layout_spaces() {
    let cases = [
        (
            "#let x = {\n  let a = 1\n\n  a\n}\n",
            "#let x = {\n  let a = 1\n\n  a\n}\n",
        ),
        ("#foo[\n  a\n\n  b\n]\n", "#foo[\n  a\n\n  b\n]\n"),
        (
            "#foo[\n/* hi\n\nthere */\n]\n",
            "#foo[\n  /* hi\n\n  there */\n]\n",
        ),
        ("/\n   :\n", "/\n  :\n"),
        ("#[/\n   : ]\n", "#[\n  /\n    :\n]\n"),
    ];
    for mode in [
        WrapMode::None,
        WrapMode::Fill,
        WrapMode::Sentence,
        WrapMode::FillSentence,
    ] {
        for width in [0, 20, 80] {
            let formatter = Typstyle::new(Config::new().with_width(width).with_wrap_mode(mode));
            for (input, expected) in cases {
                let output = formatter.format_text(input).render().unwrap();
                assert_eq!(output, expected, "mode={mode:?}, width={width}");
                assert!(output.lines().all(|line| !line.ends_with([' ', '\t'])));
                assert_eq!(formatter.format_text(&output).render().unwrap(), output);
            }
        }
    }
}

#[test]
fn source_whitespace_in_literals_is_preserved() {
    let input = "#let x = \"a \nb\"\n";
    let output = Typstyle::default().format_text(input).render().unwrap();
    assert_eq!(output, input);
}

#[test]
fn document_layout_adds_a_terminal_newline_without_indentation() {
    for input in ["text", "#let x = 1"] {
        let output = Typstyle::default().format_text(input).render().unwrap();
        assert_eq!(output, format!("{input}\n"));
        assert_eq!(
            Typstyle::default().format_text(&output).render().unwrap(),
            output
        );
    }
}

#[test]
fn partial_formatting_preserves_nested_blank_lines_without_layout_spaces() {
    let input = "#let outer = {\n  let inner = {\n    let a = 1\n\n    a\n  }\n  inner\n}\n";
    let start = input.find("{\n    let").unwrap();
    let end = input.find("\n  }\n").unwrap() + "\n  }".len();
    let formatter = Typstyle::default();
    let result = formatter
        .format_source_range(typst_syntax::Source::detached(input), start..end)
        .unwrap();

    assert_eq!(result.source_range, start..end);
    assert_eq!(result.content, &input[start..end]);
    assert!(
        result
            .content
            .lines()
            .all(|line| !line.ends_with([' ', '\t']))
    );
}
