//! Benchmarks pretty-printing of the official Typst documentation corpus
//! across default and text wrapping modes.
//!
//! Run all corpus benchmarks:
//!
//! ```text
//! cargo bench -p typstyle-core --bench typst_docs
//! ```
//!
//! Filter to the default mode (`typst-docs/all`), all wrapping modes
//! (`typst-docs/wrap/<mode>`), or a single wrapping mode:
//!
//! ```text
//! cargo bench -p typstyle-core --bench typst_docs -- 'typst-docs/all'
//! cargo bench -p typstyle-core --bench typst_docs -- 'typst-docs/wrap/'
//! cargo bench -p typstyle-core --bench typst_docs -- 'typst-docs/wrap/sentence$'
//! ```

use std::{hint::black_box, sync::LazyLock};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use typst_syntax::Source;
use typstyle_core::{Config, Typstyle, WrapMode};
use typstyle_tests::{docs_corpus, source::read_source};

/// The UTF-8 byte length of the pinned docs corpus after line-ending
/// normalization.
///
/// Criterion needs the throughput before it applies its mode or filter, so the
/// size cannot come from loading the corpus. It is verified against the loaded
/// corpus and must be updated together with [`docs_corpus::TYPST_DOCS_REV`].
const TYPST_DOCS_BYTES: u64 = 585_766;

/// Non-default text wrapping modes benchmarked alongside `typst-docs/all`.
const WRAP_MODES: [(&str, WrapMode); 3] = [
    ("fill", WrapMode::Fill),
    ("sentence", WrapMode::Sentence),
    ("fill-sentence", WrapMode::FillSentence),
];

/// The parsed docs corpus used by one benchmark iteration.
struct DocsSources {
    /// One parsed source per corpus file, in sorted path order.
    sources: Vec<Source>,
}

/// Loads and parses the whole docs corpus once, before the timer starts.
fn load_docs() -> DocsSources {
    let corpus = docs_corpus::load().expect("cannot load the Typst docs corpus for benchmarking");
    let files = corpus
        .typ_files()
        .expect("cannot enumerate the Typst docs corpus");
    assert!(!files.is_empty(), "no Typst docs found");

    let mut sources = Vec::with_capacity(files.len());
    let mut total_bytes = 0;
    for path in files {
        let source = read_source(&path)
            .unwrap_or_else(|err| panic!("cannot read `{}`: {err:#}", path.display()));
        total_bytes += source.text().len() as u64;
        sources.push(source);
    }
    assert_eq!(
        total_bytes, TYPST_DOCS_BYTES,
        "the pinned Typst docs changed size; update `TYPST_DOCS_BYTES`"
    );
    DocsSources { sources }
}

/// Formats every source in `corpus` with `formatter`, black-boxing each output.
fn format_corpus(formatter: &Typstyle, corpus: &DocsSources) {
    for source in &corpus.sources {
        black_box(
            formatter
                .format_source(source.clone())
                .render()
                .expect("expect errorless"),
        );
    }
}

/// Benchmarks formatting of the whole official Typst documentation corpus.
///
/// Each file is formatted independently within one Criterion iteration, matching
/// the document boundaries Typstyle sees in normal use. Criterion invokes the
/// closure only when a benchmark is selected, so discovery and filtered runs do
/// not fetch the corpus.
fn benchmark_typst_docs(c: &mut Criterion) {
    let corpus = LazyLock::new(load_docs);

    let mut group = c.benchmark_group("typst-docs");
    group.throughput(Throughput::Bytes(TYPST_DOCS_BYTES));

    let default_formatter = Typstyle::new(Config::new());
    group.bench_function("all", |b| {
        let corpus = &*corpus;
        b.iter(|| format_corpus(&default_formatter, corpus))
    });

    for (mode_name, mode) in WRAP_MODES {
        let formatter = Typstyle::new(Config::new().with_wrap_mode(mode));
        group.bench_function(BenchmarkId::new("wrap", mode_name), |b| {
            let corpus = &*corpus;
            b.iter(|| format_corpus(&formatter, corpus))
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_typst_docs
}
criterion_main!(benches);
