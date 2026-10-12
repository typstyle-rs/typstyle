//! Sentence decisions depend on prose context rather than syntax-leaf boundaries.

use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode};

fn assert_sentence_modes(input: &str, expected: &str) {
    for mode in [WrapMode::Sentence, WrapMode::FillSentence] {
        let formatter = Typstyle::new(Config::new().with_wrap_mode(mode));
        let actual = formatter.format_text(input).render().unwrap();
        assert_eq!(actual, expected, "{mode:?}: {input:?}");
        assert_eq!(formatter.format_text(&actual).render().unwrap(), actual);
    }
}

#[test]
fn abbreviation_policy_filters_contextual_candidates() {
    for input in [
        "A e.g. B",
        "Name Dr. Dre",
        "Time approx. 10",
        "Name (DR. Dre)",
        "Name *Dr.* Dre",
        "Name _Dr._ Dre",
        "Name *_Dr._* Dre",
        "I bought fruit etc. Next sentence.",
        "I bought fruit etc. *Next* sentence.",
    ] {
        assert_sentence_modes(input, &format!("{input}\n"));
    }
}

#[test]
fn continuation_context_crosses_inline_markup_and_quotes() {
    for input in [
        "Hello world. next part.",
        "Hello *world.* next part.",
        "Hello *_world._* next part.",
        "Hello world. *next* part.",
        "He said \"stop.\" then left.",
        "He said 'stop.' then left.",
        "Hello world. , next.",
        "Hello *world.* , next.",
        "We use tools, e.g., typstyle.",
        "For example (i.e., this feature).",
    ] {
        assert_sentence_modes(input, &format!("{input}\n"));
    }
}

#[test]
fn terminal_punctuation_is_not_hidden_by_numeric_or_abbreviation_shapes() {
    for (input, expected) in [
        (
            "The value is 1.2. Next sentence.",
            "The value is 1.2.\nNext sentence.\n",
        ),
        (
            "Version 1.2.3. Next sentence.",
            "Version 1.2.3.\nNext sentence.\n",
        ),
        (
            "He lives in the U.S. Next sentence.",
            "He lives in the U.S.\nNext sentence.\n",
        ),
        ("Name Dr.! Next.", "Name Dr.!\nNext.\n"),
        (
            "The identifier is WidgetDr. Next sentence.",
            "The identifier is WidgetDr.\nNext sentence.\n",
        ),
        (
            "He said \"stop.\" Then left.",
            "He said \"stop.\"\nThen left.\n",
        ),
    ] {
        assert_sentence_modes(input, expected);
    }
}

#[test]
fn literal_atoms_supply_context_without_editing_their_interior() {
    assert_sentence_modes(
        "He said `stop.,` then left.",
        "He said `stop.,` then left.\n",
    );
    assert_sentence_modes(
        "He said `stop. Next.` then left. Next.",
        "He said `stop. Next.` then left.\nNext.\n",
    );
    assert_sentence_modes("Hello $arrow.r.long$ Next.", "Hello $arrow.r.long$ Next.\n");
    assert_sentence_modes("Hello $x.$ Next.", "Hello $x.$\nNext.\n");
}

#[test]
fn protected_joins_survive_sentence_segmentation() {
    for join in ['\u{00a0}', '\u{202f}', '\u{2060}'] {
        let input = format!("Hello world.{join}Next part.");
        assert_sentence_modes(&input, &format!("{input}\n"));
        let input = format!("你好。{join}世界。");
        assert_sentence_modes(&input, &format!("{input}\n"));
    }
    assert_sentence_modes("Hello world.~Next part.", "Hello world.~Next part.\n");
    assert_sentence_modes("你好。~世界。", "你好。~世界。\n");
}

#[test]
fn metadata_stays_attached_before_the_sentence_break() {
    assert_sentence_modes("Hello.<label> Next.", "Hello.<label>\nNext.\n");
    assert_sentence_modes("Hello. /* note */ Next.", "Hello. /* note */\nNext.\n");
    // Without a following whitespace there is no editable position, so the break is
    // not materialized and the metadata cannot be separated from its sentence.
    assert_sentence_modes("Hello. <label>Next.", "Hello. <label>Next.\n");
    // A block comment is not line-sensitive, so the break can land before it.
    assert_sentence_modes("Hello. /* note */Next.", "Hello.\n/* note */Next.\n");
    // A source linebreak already provides the break, so no extra blank line appears.
    assert_sentence_modes("Hello.\n<label>Next.", "Hello.\n<label>Next.\n");
}

#[test]
fn editable_whitespace_at_wrapper_edges_uses_the_enclosing_plan() {
    assert_sentence_modes("Hello *world. *Next.", "Hello *world.\n*Next.\n");
    assert_sentence_modes("Hello.* Next.*", "Hello.*\nNext.*\n");
    assert_sentence_modes("Hello.* *Next.", "Hello.*\n*Next.\n");
}

#[test]
fn sentence_decisions_cross_soft_source_newlines() {
    let input = "Hello *world.*\nnext part.\nNext sentence.\n";
    let sentence = Typstyle::new(Config::new().with_wrap_mode(WrapMode::Sentence));
    assert_eq!(sentence.format_text(input).render().unwrap(), input);
    let fill = Typstyle::new(Config::new().with_wrap_mode(WrapMode::FillSentence));
    let expected = "Hello *world.* next part.\nNext sentence.\n";
    assert_eq!(fill.format_text(input).render().unwrap(), expected);
    assert_eq!(fill.format_text(expected).render().unwrap(), expected);
}

#[test]
fn fragment_endpoints_do_not_materialize_outer_sentence_layout() {
    let input = "Before. *world.* next part.";
    let start = input.find('*').unwrap();
    let end = input.rfind('*').unwrap() + 1;
    let formatter = Typstyle::new(Config::new().with_wrap_mode(WrapMode::Sentence));
    let result = formatter
        .format_source_range(Source::detached(input), start..end)
        .unwrap();
    assert_eq!(result.source_range, start..end);
    assert_eq!(result.content, "*world.*");
}

#[test]
fn sentence_boundaries_are_independent_of_fill_width() {
    for width in [10, 20, 80] {
        let formatter = Typstyle::new(
            Config::new()
                .with_width(width)
                .with_wrap_mode(WrapMode::FillSentence),
        );
        let output = formatter
            .format_text("Name *Dr.* Dre arrived. Next sentence.")
            .render()
            .unwrap();
        assert!(output.contains("arrived.\nNext"), "{width}: {output:?}");
        assert_eq!(formatter.format_text(&output).render().unwrap(), output);
    }
    assert_sentence_modes("One!Two.", "One!Two.\n");
    assert_sentence_modes("A.#foo Next.", "A.#foo Next.\n");
}

#[test]
fn cjk_sentence_boundaries_allow_unspaced_prose() {
    for (input, expected) in [
        ("你好。世界。", "你好。\n世界。\n"),
        ("你好。 世界。", "你好。\n世界。\n"),
        ("中文。Dr. Wang arrived.", "中文。\nDr. Wang arrived.\n"),
        ("中文｡Dr. Wang arrived.", "中文｡\nDr. Wang arrived.\n"),
        (
            "「こんにちは！」次の文です。",
            "「こんにちは！」\n次の文です。\n",
        ),
        ("*你好。世界。*", "*你好。\n世界。*\n"),
    ] {
        assert_sentence_modes(input, expected);
    }
}

#[test]
fn cjk_breaks_keep_line_sensitive_markers_inline() {
    for marker in ["+", "-", "/", "=="] {
        let input = format!("你好。{marker} 正文。");
        assert_sentence_modes(&input, &format!("{input}\n"));
    }
    for marker in ["1.", "+1."] {
        let input = format!("你好。{marker} 正文。");
        assert_sentence_modes(&input, &format!("你好。{marker}\n正文。\n"));
    }
    assert_sentence_modes("你好。12月开始。", "你好。\n12月开始。\n");
    assert_sentence_modes("你好。==正文。", "你好。\n==正文。\n");
}

#[test]
fn unspaced_cjk_boundaries_start_nodes_on_new_lines() {
    // A CJK sentence ending needs no whitespace. The break starts the next node,
    // or its enclosing wrapper, on a new line instead of splitting the wrapper.
    for (input, expected) in [
        ("*你好。*世界。", "*你好。*\n世界。\n"),
        ("你好。*世界。*", "你好。\n*世界。*\n"),
        ("你好。*_世界。_*", "你好。\n*_世界。_*\n"),
        ("你好。`世界。`", "你好。\n`世界。`\n"),
        ("你好。$x.$", "你好。\n$x.$\n"),
        ("你好。<label>世界。", "你好。<label>\n世界。\n"),
        ("你好。*\"世界。\"*", "你好。\n*\"世界。\"*\n"),
        ("你好。*$x.$*", "你好。\n*$x.$*\n"),
        ("你好。<label>*世界。*", "你好。<label>\n*世界。*\n"),
    ] {
        assert_sentence_modes(input, expected);
    }
}

#[test]
fn halfwidth_cjk_terminator_ends_prose_before_opaque_markup() {
    for input in ["こんにちは｡#emph[次の文｡]", "こんにちは｡ #emph[次の文｡]"]
    {
        assert_sentence_modes(input, "こんにちは｡\n#emph[次の文｡]\n");
    }
}

#[test]
fn attached_linebreak_does_not_break_preceding_wrapper_or_metadata_space() {
    assert_sentence_modes(
        "*Sentence 1. *\\ Sentence 2.",
        "*Sentence 1. * \\\nSentence 2.\n",
    );
    assert_sentence_modes(
        "Sentence 1. /* note */\\ Sentence 2.",
        "Sentence 1. /* note */ \\\nSentence 2.\n",
    );
    assert_sentence_modes(
        "Sentence 1. <label>\\ Sentence 2.",
        "Sentence 1. <label> \\\nSentence 2.\n",
    );
}

#[test]
fn wrapper_starting_with_opaque_node_does_not_leak_to_following_nodes() {
    assert_sentence_modes(
        "你好。*#foo()*世界。下一句。",
        "你好。\n*#foo()*世界。\n下一句。\n",
    );
    assert_sentence_modes("你好。*_#foo()_ 世界。*", "你好。\n*_#foo()_ 世界。*\n");
    assert_sentence_modes(
        "你好。* // note\n世界。*下一句。",
        "你好。*\n// note\n世界。*\n下一句。\n",
    );
}

#[test]
fn disabled_markup_keeps_its_literal_contents() {
    let input = "/* @typstyle off */ *First.  Next.* following text.\n";
    assert_sentence_modes(input, input);
}
