#import "../book.typ": *
#import callout: *

#show: book-page.with(title: "Rust Development")

This section covers building, testing, and benchmarking the Rust crates of Typstyle.

= Building Components

== CLI and Core

```bash
cargo build                     # Debug build
cargo build --release           # Release build
cargo build -p typstyle         # CLI only
cargo build -p typstyle-core    # Core only
```

= Running Tests

#important[
  Update snapshots with `cargo insta` when changing core library or fixtures.

  Add CLI tests to `crates/typstyle/tests/` when modifying CLI behavior.
  For style arguments, add tests to `test_style_args.rs`.
]

== Test Commands

List all tests:
```bash
cargo nextest list --workspace
```

Run all tests and review snapshots:
```bash
cargo nextest run --workspace --no-fail-fast
cargo insta review
```

Snapshot tests only:
```bash
cargo nextest run --workspace -E 'test([snapshot])' \
  --no-fail-fast --no-default-features
cargo insta review
```

End-to-end tests only:
```bash
cargo nextest run --workspace -E 'test([e2e])' --no-fail-fast
```

These cover real-world Typst repositories and the official Typst documentation. The docs corpus is a sparse checkout of the Typst repository at `TYPST_DOCS_REV` (see `tests/src/docs_corpus.rs`), fetched on first use and cached under `tests/.cache/docs`, so the first run needs `git` and network access.

Exclude end-to-end tests:
```bash
cargo nextest run --workspace -E '!test([e2e])' --no-fail-fast
```

CLI tests only:
```bash
cargo nextest run -p typstyle --no-fail-fast
```

Integration tests:
```bash
cargo nextest run -p tests --no-fail-fast
```

== Snapshot Management

Review and accept snapshot changes:
```bash
cargo insta review    # Interactive review
cargo insta accept    # Accept all changes
```

== Benchmarks

Using #link("https://github.com/bheisler/criterion.rs")[Criterion.rs]:

```bash
cargo bench --workspace -- --list          # List benchmarks
cargo bench --workspace                    # Run all benchmarks
cargo bench -p typstyle-core --bench pretty_print
cargo bench -p typstyle-core --bench typst_docs
cargo bench -p typstyle-core --bench wrap_stress
```

The `typst_docs` benchmark formats the same pinned Typst documentation corpus as the docs end-to-end test across default (`typst-docs/all`) and text wrapping modes (`typst-docs/wrap/*`). It is fetched on first use and cached under `tests/.cache/docs`; when bumping `TYPST_DOCS_REV`, update `TYPST_DOCS_BYTES` in `crates/typstyle-core/benches/typst_docs.rs` too. The `wrap_stress` target is excluded from automated CI benchmark runs and is intended for on-demand local scaling checks.

View HTML reports at `target/criterion/report/index.html`.

= Code Quality

== Formatting and Linting

Format code:
```bash
cargo fmt --all
```

Run clippy:
```bash
cargo clippy --workspace --all-targets --all-features
```

== Testing Guidelines

- Add tests to appropriate modules when implementing new features
- Use snapshot tests for formatter output validation
- Update snapshots after intentional changes to formatting behavior
- Add CLI integration tests for new command-line features
- Benchmark performance-critical changes
