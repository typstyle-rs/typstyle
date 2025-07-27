#import "./book.typ": *
#import "../deps.typ": *

#show: book-page.with(title: "Developer Guide")

This guide covers environment setup, building, testing, and documentation for typstyle.

= Prerequisites

- Rust stable toolchain with #link("https://doc.rust-lang.org/cargo/")[cargo]
- Node.js and #link("https://yarnpkg.com/")[yarn] (for web assets)
- #link("https://nexte.st/")[cargo-nextest] and #link("https://insta.rs/")[cargo-insta] (for testing)
- #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa] (for documentation)

= Initial Setup

Clone and build the project:

```bash
git clone https://github.com/typstyle-rs/typstyle.git
cd typstyle
cargo build              # Debug build
cargo build --release    # Release build
```

Install required tools:
```bash
# For testing
cargo binstall cargo-nextest cargo-insta
# For building wasm
cargo binstall wasm-pack
# For documentation
cargo binstall shiroa
```

= Workspace Layout

- `crates/typstyle/` — formatter CLI
- `crates/typstyle-core/` — core formatting logic
- `crates/typstyle-consistency/` — consistency test framework
- `crates/typstyle-typlugin/` — typst plugin for embedded usage
- `crates/typstyle-wasm/` — wasm bindings
- `tests/` — integration tests and fixtures
- `docs/` — documentation source (based on shiroa and written in typst)
- `contrib/typstyle-embedded/` — typstyle as typst package
- `playground/` — web-based interactive playground
