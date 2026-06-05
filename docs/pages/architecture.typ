#import "./book.typ": *

#show: book-page.with(title: "Architecture")

= High Level Overview

Typstyle is a code formatter: its input is a string of Typst source code and its output is a string of formatted Typst source code.

To format code, Typstyle follows a pipeline of five steps:

+ *Parsing*: The input is parsed into an Abstract Syntax Tree (AST) using the `typst-syntax` package. If the input contains syntax errors, formatting is skipped and the original code is returned unchanged.
+ *Attach Attributes*: The AST is traversed and metadata is attached to nodes — marking regions to skip, detecting multiline intent, and tagging structures that need special handling.
+ *Formatting*: The attributed AST is converted into a Wadler-style pretty-print document tree, which is then rendered into a formatted string.
+ *Post Processing*: Final cleanup is applied to ensure consistent file endings and remove artifacts.
+ *Output*: The formatted code is returned.

Steps 2 and 3 are where the core formatting logic lives.

= Pipeline in Detail

== Parsing

Typstyle uses the #link("https://crates.io/crates/typst-syntax")[`typst-syntax`] crate — the same parser used by the Typst compiler — to produce the AST. This ensures full fidelity with Typst's own grammar and semantics.

If the input contains syntax errors, Typstyle returns the original source unmodified. There is no partial or best-effort formatting for invalid code.

== Attribute Attachment

Before formatting, Typstyle walks the AST and attaches processing metadata:

- Regions protected by escape hatch directives are marked to be skipped.
- Structures (arrays, dictionaries, function parameters, etc.) are checked for *multiline flavor*: if the user already placed a line break before the first item, the structure will never be collapsed back to a single line.

== Formatting (Wadler's Pretty Printer)

This is the heart of Typstyle. The attributed AST is transformed into an intermediate *pretty-print document* using Philip Wadler's algorithm, which optimally decides where to insert line breaks while respecting the configured line width.

The document tree encodes layout choices — flat vs. multiline via grouping, indentation depth via nesting, and alternative layouts via choice combinators. Once complete, the tree is rendered to a string with consistent indentation and line breaks.

== Post Processing

After rendering, Typstyle applies final normalization to produce a clean, consistent output.

= Crate Architecture

Typstyle is organized as a Cargo workspace with several crates:

- *`typstyle-core`*: The core formatting engine — parsing, attribute attachment, Wadler-style pretty printing, and post processing. All formatting logic lives here.
- *`typstyle`*: The CLI binary. Handles argument parsing, file discovery (including recursive directory walking), stdin/stdout I/O, and check/diff modes.
- *`typstyle-wasm`*: WebAssembly bindings that expose the core formatter to JavaScript runtimes. Used by the web playground and NPM package.
- *`typstyle-typlugin`*: A Typst plugin compiled to WASM, enabling Typstyle to format code from within Typst documents (used by the documentation site for live examples).
- *`typstyle-consistency`*: A test harness for verifying correctness. Compares rendered outputs (via pixel-level image diffs) and checks error consistency before and after formatting.
