//! Snapshot tests for formatting only a part of a fixture file.
//!
//! Uses [`Typstyle::format_source_range`] to reformat selected byte ranges.

use std::{ops::Range, path::Path};

use insta::internals::Content;
use libtest_mimic::{Failed, Trial};
use typstyle_core::Typstyle;
use typstyle_tests::common::{fixtures_dir, read_source};

/// A partial-formatting test case: a fixture file and the byte ranges to format in it.
struct Testcase {
    /// The fixture file path relative to the fixtures directory.
    path: &'static str,
    /// The byte ranges of the file to format, one test per range.
    ranges: Vec<Range<usize>>,
}

impl Testcase {
    /// Creates a test case for the given fixture path and byte ranges.
    pub fn new(path: &'static str, ranges: impl IntoIterator<Item = Range<usize>>) -> Testcase {
        Self {
            path,
            ranges: ranges.into_iter().collect(),
        }
    }
}

/// Creates one test for each `.typ` file in the current directory or
/// sub-directories of the current directory.
pub fn collect_tests() -> Vec<Trial> {
    let cases = [
        Testcase::new(
            "partial/indenta.typ",
            [80..85, 200..300, 387..713, 867..869],
        ),
        Testcase::new(
            "partial/erroneous.typ",
            [5..10, 17..20, 25..30, 36..40, 53..56],
        ),
    ];

    let root = fixtures_dir();
    cases
        .into_iter()
        .flat_map(|case| {
            let path = root.join(case.path);
            case.ranges.into_iter().map(move |rng| {
                let path = path.clone();
                Trial::test(
                    format!("{} - {}..{}", case.path, rng.start, rng.end),
                    move || check_snapshot(&path, rng),
                )
                .with_kind("partial")
            })
        })
        .collect()
}

/// Formats a byte range of a fixture file and asserts the output against its snapshot.
///
/// The snapshot records the original range text and the formatted result. Ranges that
/// fail to format are marked as erroneous in the snapshot info and snapshotted as is.
fn check_snapshot(path: &Path, range: Range<usize>) -> Result<(), Failed> {
    let source = read_source(path)?;

    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings.set_snapshot_path(path.parent().unwrap().join("snap"));
    settings.set_input_file(path);

    let snap_name = format!(
        "{}-{}_{}",
        path.file_name().unwrap().to_str().unwrap(),
        range.start,
        range.end
    );
    let mut info: Vec<(Content, Content)> = vec![("range".into(), range_to_content(&range))];

    match Typstyle::default().format_source_range(source.clone(), range.clone()) {
        Ok(result) => {
            info.push(("range_node".into(), range_to_content(&result.source_range)));
            settings.set_raw_info(&Content::Map(info));

            settings.bind(|| {
                let snap = format!(
                    "{}\n---\n{}",
                    &source.text()[result.source_range],
                    result.content
                );
                insta::assert_snapshot!(snap_name, snap);
            });
        }
        Err(_) => {
            info.push(("erroneous".into(), true.into()));
            settings.set_raw_info(&Content::Map(info));

            settings.bind(|| {
                insta::assert_snapshot!(snap_name, &source.text()[range]);
            });
        }
    }
    Ok(())
}

/// Converts a byte range to an insta [`Content`] map with `start` and `end` entries.
fn range_to_content(range: &Range<usize>) -> Content {
    Content::Map(vec![
        ("start".into(), (range.start as u64).into()),
        ("end".into(), (range.end as u64).into()),
    ])
}
