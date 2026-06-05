use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use typstyle_consistency::{CheckingOptions, ErrorSink, FormattedSources, FormatterHarness};

fn compare_with_formatter(
    source: &str,
    formatted_source: &str,
    options: CheckingOptions,
) -> Result<ErrorSink> {
    compare_with_formatter_fn(source, options, |_| Ok(formatted_source.to_string()))
}

fn compare_with_formatter_fn(
    source: &str,
    options: CheckingOptions,
    formatter: impl Fn(typst::syntax::Source) -> Result<String>,
) -> Result<ErrorSink> {
    let main_path = Path::new("main.typ");
    let mut sink = ErrorSink::new("broken formatter test".to_string());
    let mut harness = FormatterHarness::new("broken formatter".to_string(), PathBuf::new())?;
    harness.add_source_file(main_path, source)?;

    let base_world = harness.snapshot();
    let formatted = FormattedSources {
        name: "broken".to_string(),
        sources: harness.format(&base_world, formatter, &mut sink)?,
    };

    harness.compile_and_compare([formatted].iter(), main_path, options, &mut sink)?;

    Ok(sink)
}

#[track_caller]
fn assert_ok(sink: ErrorSink) {
    assert!(
        sink.is_ok(),
        "formatter was reported as broken unexpectedly:\n{sink}"
    );
}

#[track_caller]
fn assert_detected(sink: ErrorSink, expected_message: &str) -> String {
    let output = format!("{sink}");
    assert!(
        !sink.is_ok(),
        "broken formatter was not detected; expected message containing `{expected_message}`"
    );
    assert!(
        output.contains(expected_message),
        "diagnostics did not contain `{expected_message}`:\n{output}"
    );
    output
}

#[test]
fn accepts_unchanged_formatted_source() -> Result<()> {
    let sink = compare_with_formatter("Hello\n", "Hello\n", Default::default())?;

    assert_ok(sink);
    Ok(())
}

#[test]
fn detects_formatter_returning_error() -> Result<()> {
    let sink = compare_with_formatter_fn("Hello\n", Default::default(), |_| -> Result<String> {
        bail!("formatter crashed")
    })?;

    let output = assert_detected(sink, "failed to format file");
    assert!(
        output.contains("formatter crashed"),
        "diagnostics did not contain the formatter error:\n{output}"
    );
    Ok(())
}

#[test]
fn detects_formatted_source_that_no_longer_evaluates() -> Result<()> {
    let sink = compare_with_formatter("Hello\n", "#missing_binding\n", Default::default())?;

    assert_detected(sink, "Failed to evaluate formatted document");
    Ok(())
}

#[test]
fn detects_formatted_source_that_fails_during_compile() -> Result<()> {
    let sink = compare_with_formatter(
        "Hello\n",
        "#context panic(\"compile-only failure\")\n",
        Default::default(),
    )?;

    assert_detected(sink, "Formatted doc failed to compile.");
    Ok(())
}

#[test]
fn detects_formatter_masking_original_compile_error() -> Result<()> {
    let sink = compare_with_formatter(
        "#context panic(\"original compile failure\")\n",
        "Hello\n",
        Default::default(),
    )?;

    assert_detected(sink, "Original doc failed to compile.");
    Ok(())
}

#[test]
fn detects_formatter_masking_original_eval_error() -> Result<()> {
    let sink = compare_with_formatter("#missing_binding\n", "Hello\n", Default::default())?;

    assert_detected(sink, "Failed to evaluate original document");
    Ok(())
}

#[test]
fn detects_content_changes_when_strict_content_equality_is_enabled() -> Result<()> {
    let sink = compare_with_formatter(
        "$ 1+1 $",
        "$ 1 + 1 $",
        CheckingOptions {
            strict_content_equality: true,
            ..Default::default()
        },
    )?;

    assert_detected(
        sink,
        "Content differs between original and formatted documents",
    );
    Ok(())
}

#[test]
fn detects_rendered_page_changes() -> Result<()> {
    let sink = compare_with_formatter("Hello\n", "World\n", Default::default())?;

    assert_detected(sink, "Page 1/1 differs by");
    Ok(())
}

#[test]
fn detects_page_count_changes() -> Result<()> {
    let sink = compare_with_formatter(
        "#set page(width: 160pt, height: 80pt)\nHello\n",
        "#set page(width: 160pt, height: 80pt)\nHello\n#pagebreak()\nWorld\n",
        Default::default(),
    )?;

    assert_detected(sink, "Page count mismatch");
    Ok(())
}

#[test]
fn detects_document_metadata_changes() -> Result<()> {
    let sink = compare_with_formatter(
        "#set document(title: \"Original\")\nHello\n",
        "#set document(title: \"Formatted\")\nHello\n",
        Default::default(),
    )?;

    assert_detected(sink, "The titles are not consistent");
    Ok(())
}

#[test]
fn detects_page_metadata_changes() -> Result<()> {
    let sink = compare_with_formatter(
        "#set page(numbering: \"1\")\nHello\n",
        "#set page(numbering: \"i\")\nHello\n",
        Default::default(),
    )?;

    assert_detected(sink, "The numbering of page 0 differs.");
    Ok(())
}

#[test]
fn detects_inconsistent_evaluation_error_messages() -> Result<()> {
    let sink = compare_with_formatter("#first_missing\n", "#second_missing\n", Default::default())?;

    assert_detected(
        sink,
        "The error messages are not consistent after formatting",
    );
    Ok(())
}

#[test]
fn detects_inconsistent_compile_error_messages() -> Result<()> {
    let sink = compare_with_formatter(
        "#context panic(\"before formatting\")\n",
        "#context panic(\"after formatting\")\n",
        Default::default(),
    )?;

    assert_detected(
        sink,
        "The error messages are not consistent after formatting",
    );
    Ok(())
}

#[test]
fn detects_expected_compile_success_when_both_versions_fail_to_evaluate() -> Result<()> {
    let sink = compare_with_formatter(
        "#missing_binding\n",
        "#missing_binding\n",
        CheckingOptions {
            expect_compile_success: true,
            ..Default::default()
        },
    )?;

    assert_detected(sink, "Both docs failed to compile.");
    Ok(())
}

#[test]
fn detects_expected_compile_success_when_both_versions_fail_during_compile() -> Result<()> {
    let sink = compare_with_formatter(
        "#context panic(\"before formatting\")\n",
        "#context panic(\"after formatting\")\n",
        CheckingOptions {
            expect_compile_success: true,
            ..Default::default()
        },
    )?;

    assert_detected(sink, "Both docs failed to compile.");
    Ok(())
}

#[test]
fn detects_expected_error_when_both_versions_succeed() -> Result<()> {
    let sink = compare_with_formatter(
        "Hello\n",
        "Hello\n",
        CheckingOptions {
            expect_error: true,
            ..Default::default()
        },
    )?;

    assert_detected(
        sink,
        "Both docs evaluated successfully, but were expected to fail.",
    );
    Ok(())
}

#[test]
fn detects_expected_error_when_changed_versions_compile() -> Result<()> {
    let sink = compare_with_formatter(
        "Hello\n",
        "World\n",
        CheckingOptions {
            expect_error: true,
            ..Default::default()
        },
    )?;

    assert_detected(
        sink,
        "Both docs compiled successfully, but were expected to fail.",
    );
    Ok(())
}
