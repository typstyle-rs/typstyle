# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**typstyle** is a beautiful and reliable code formatter for [Typst](https://typst.app/). It's built in Rust and uses Wadler's pretty printing algorithm to consistently format Typst source code.

## Architecture

The project is organized as a Cargo workspace with these main crates:

- **`typstyle-core`**: Core formatting logic with Wadler's pretty printer
- **`typstyle`**: CLI application built on top of core
- **`typstyle-wasm`**: WebAssembly bindings for web usage
- **`typstyle-typlugin`**: Typst plugin (WASM target)
- **`typstyle-consistency`**: Testing utilities for convergence tests

## Development Commands

### Build & Test
```bash
cargo build                    # Build all crates
cargo test                     # Run all tests
cargo test -p typstyle-core    # Run specific crate tests
cargo test --test unit         # Run unit tests only
cargo test --test e2e          # Run end-to-end tests
```

### CLI Usage
```bash
cargo run -p typstyle -- --help              # Show CLI help
cargo run -p typstyle -- -i file.typ         # Format file in place
cargo run -p typstyle -- --check file.typ    # Check formatting
cargo run -p typstyle -- --diff file.typ     # Show diff
```

### Documentation & Development
```bash
just dev-docs           # Serve documentation locally
just build-docs         # Build documentation
just build-plugin       # Build typst plugin WASM
just generate-cli-help  # Update CLI help text
```

### Testing
The project uses comprehensive testing:
- **Snapshot tests**: Use `insta` crate, outputs in `tests/fixtures/*/snap/`
- **Convergence tests**: Ensure formatting converges (format twice = same result)
- **Correctness tests**: Ensure rendered output unchanged after formatting
- **E2E tests**: Test against real repos like `tablex`, `cetz`, `fletcher`

### Benchmarking
```bash
cargo bench -p typstyle-core    # Run benchmarks
cargo bench --bench pretty_print # Pretty printing benchmarks
```

## Key Files

- **Entry points**: `crates/typstyle/src/main.rs` (CLI), `crates/typstyle-core/src/lib.rs` (core)
- **Formatting logic**: `crates/typstyle-core/src/pretty/` contains formatting rules
- **Configuration**: `crates/typstyle-core/src/config.rs`
- **Tests**: `tests/` directory with unit, e2e, and fixtures

## Formatting Features

- **Opinionated**: Consistent style across codebases
- **Code-only**: Formats code, preserves content formatting
- **Convergence**: Running formatter twice produces same result
- **Correctness**: Preserves rendered output appearance

## Development Notes

- Requires Rust 1.83+
- Uses `just` for task automation
- Web playground available at `/playground/` directory
- Documentation uses shiroa for static site generation