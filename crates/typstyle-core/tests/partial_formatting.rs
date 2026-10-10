use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode};

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
