# AGENTS.md

**Typstyle** is an opinionated, code-only formatter for [Typst](https://typst.app/), written in Rust using Wadler's pretty-printing algorithm.

## Invariants

- **Idempotency**: Formatting twice must produce identical output.
- **Correctness**: Formatting must preserve rendered output.
- **Graceful fallback**: Return syntax-invalid input unchanged.
- **Literal whitespace**: Preserve whitespace inside strings, raw blocks, and `@typstyle off` regions.

## Documentation

Read the relevant doc before working in that area:

- [README.md](README.md) — overview, CLI usage, design goals
- [docs/pages/architecture.typ](docs/pages/architecture.typ) — formatting pipeline and crate roles
- [docs/pages/dev-guide.typ](docs/pages/dev-guide.typ) — prerequisites and workspace layout
- [docs/pages/dev-guide/core.typ](docs/pages/dev-guide/core.typ) — building, testing, `insta` snapshots, benchmarks
- [docs/pages/dev-guide/formatter-options.typ](docs/pages/dev-guide/formatter-options.typ) — cross-crate checklist for option changes
- [docs/pages/dev-guide/documentation.typ](docs/pages/dev-guide/documentation.typ) — docs site, embedded plugin, CLI help sync
- [docs/pages/dev-guide/playground.typ](docs/pages/dev-guide/playground.typ) — web playground and WASM workflow
- [docs/pages/dev-guide/release-process.typ](docs/pages/dev-guide/release-process.typ) — versioning, changelog, releases
- [justfile](justfile), [Cargo.toml](Cargo.toml), [.github/workflows/](.github/workflows/) — task recipes, workspace config/lints, CI

## Guidelines

- **Cross-boundary options**: Follow [docs/pages/dev-guide/formatter-options.typ](docs/pages/dev-guide/formatter-options.typ) whenever adding or changing formatter options.
- **Generated artifacts**: Sync the `README.md` CLI help via its script rather than hand-editing it after CLI changes, and rebuild WASM bindings/plugins (per the linked guides) when checking docs or the playground.
- **Testing & verification**: Run the relevant checks from [justfile](justfile) and [.github/workflows/](.github/workflows/). Prefer `--no-default-features` for quick test validation; run consistency tests (default features) only when a change could cause a consistency regression or when requested. Do not add `relax_convergence` or `skip_consistency` directives to silence failures without justification.
- **Snapshots**: Avoid interactive `cargo insta review` in non-interactive shells, never hand-edit `.snap` files, and inspect every diff before accepting (`cargo insta accept`)—treat unexpected diffs outside scope as regressions.
