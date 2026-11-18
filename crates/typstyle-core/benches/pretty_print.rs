//! Benchmarks pretty-printing of the test assets with the default configuration.
//!
//! Run every benchmark:
//!
//! ```text
//! cargo bench -p typstyle-core --bench pretty_print
//! ```
//!
//! Criterion treats the positional argument as a substring filter on the
//! benchmark ID. Run a group or a single case with:
//!
//! ```text
//! cargo bench -p typstyle-core --bench pretty_print -- 'wrap/'
//! cargo bench -p typstyle-core --bench pretty_print -- 'wrap/sentence-cjk'
//! cargo bench -p typstyle-core --bench pretty_print -- 'wrap/sentence-latin/fill-sentence'
//! ```
//!
//! The wrap-mode IDs are `wrap/<document>/<mode>`, where `<mode>` is one of
//! `none`, `fill`, `sentence`, or `fill-sentence`.
//!
//! For a quick pass while iterating, shorten the run:
//!
//! ```text
//! cargo bench -p typstyle-core --bench pretty_print -- 'wrap/markup-general' \
//!     --sample-size 10 --measurement-time 1 --warm-up-time 1
//! ```
//!
//! Results are stored under `target/criterion/`; add `--noplot` to skip plot
//! generation.

use std::fs;

use criterion::{Criterion, criterion_group, criterion_main};
use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode};

fn bench_pretty(c: &mut Criterion, id: &str, path: &str, config: Config) {
    let content = fs::read_to_string(path).unwrap();
    let source = Source::detached(content);
    let t = Typstyle::new(config);

    c.bench_function(id, |b| {
        b.iter(|| {
            t.format_source(source.clone())
                .render()
                .expect("expect errorless")
        })
    });
}

/// (path, name)
const TEST_ASSETS: &[(&str, &str)] = &[
    ("articles/undergraduate-math", "undergraduate-math"),
    ("articles/_cpe", "cpe"),
    ("packages/cetz-manual", "cetz-manual"),
    ("packages/codly", "codly"),
    ("packages/fletcher-diagram", "fletcher-diagram"),
    ("packages/fletcher-draw", "fletcher-draw"),
    ("packages/tablex", "tablex"),
    ("packages/touying/core", "touying-core"),
    ("packages/touying/utils", "touying-utils"),
    ("unit/code/perf-nest", "deep-nested-args"),
    (
        "ai/feat/comprehensive-showcase",
        "ai-comprehensive-showcase",
    ),
];

fn benchmark_pretty(c: &mut Criterion) {
    for (path, name) in TEST_ASSETS {
        bench_pretty(
            c,
            &format!("pretty-{name}"),
            &format!("../../tests/fixtures/{path}.typ"),
            if *name == "deep-nested-args" {
                Config::new().with_width(10) // special config
            } else {
                Config::new()
            },
        );
    }
}

/// Text wrapping modes to compare.
const WRAP_MODES: [(&str, WrapMode); 4] = [
    ("none", WrapMode::None),
    ("fill", WrapMode::Fill),
    ("sentence", WrapMode::Sentence),
    ("fill-sentence", WrapMode::FillSentence),
];

/// Prose-dense fixtures. Code- or math-heavy documents barely exercise wrapping,
/// so they make the modes look identical.
const PROSE_DOCS: &[(&str, &str)] = &[
    ("packages/cetz-manual", "cetz-manual"),
    ("unit/markup/reflow/general", "markup-general"),
    ("unit/markup/reflow/sentence-latin", "sentence-latin"),
    ("unit/markup/reflow/sentence-cjk", "sentence-cjk"),
];

/// Repeat each fixture to this size so parsing and layout noise is amortized and
/// the wrapping mode dominates the measurement.
const TARGET_BYTES: usize = 256 * 1024;

fn load_large(path: &str) -> String {
    let content = fs::read_to_string(format!("../../tests/fixtures/{path}.typ")).unwrap();
    let mut text = String::with_capacity(TARGET_BYTES + content.len());
    while text.len() < TARGET_BYTES {
        text.push_str(&content);
        if !text.ends_with('\n') {
            text.push('\n');
        }
    }
    text
}

/// Benchmarks every text wrapping mode on large, prose-dense documents.
fn benchmark_wrap_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("wrap");
    for (path, name) in PROSE_DOCS {
        let source = Source::detached(load_large(path));
        for (mode_name, mode) in WRAP_MODES {
            let t = Typstyle::new(Config::new().with_wrap_mode(mode));
            group.bench_function(format!("{name}/{mode_name}"), |b| {
                b.iter(|| {
                    t.format_source(source.clone())
                        .render()
                        .expect("expect errorless")
                })
            });
        }
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_pretty, benchmark_wrap_modes
}
criterion_main!(benches);
