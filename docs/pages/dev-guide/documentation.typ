#import "../book.typ": *
#import callout: *

#show: book-page.with(title: "Documentation Development")

This section covers building and maintaining the typstyle documentation.

The documentation uses #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa] to build from Typst source files.

= Building Documentation

#important[
  Documentation build requires CLI help text and the typstyle plugin to be generated first. Use `just build-docs` to handle dependencies automatically, or run `just generate-cli-help` and `just build-plugin` manually before building.
]

== Development Server

Development server with auto-reload:
```bash
just dev-docs
# or manually:
shiroa serve docs/pages -w . --mode static-html
```

== Static Build

Build static documentation:
```bash
just build-docs
# or manually:
shiroa build docs/pages -w . --mode static-html
```

== Build Plugin

Build the typstyle plugin for embedded usage:
```bash
just build-plugin
# or manually:
cargo build -p typstyle-typlugin --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/typstyle_typlugin.wasm contrib/typstyle-embedded/assets/typstyle.wasm
```

== Generate CLI Help

Generate CLI help text:
```bash
just generate-cli-help
# or manually:
cargo run -p typstyle -- --help > docs/assets/generated/cli-help.txt
```

This is used for #cross-link("/cli-usage.typ")[cli-usage].

= Documentation Structure

The documentation is organized as follows:
- `docs/book.typ` — main book configuration and metadata
- `docs/pages/` — individual documentation pages
- `docs/packages/` — dependent third party packages not in universe
- `docs/templates/` — page templates and components
- `docs/assets/` — static assets and generated content

#note[
  The shiroa root directory is `docs/pages/`. For better organization, the main book configuration is located at `docs/book.typ` and imported by `docs/pages/book.typ`.
]

= Writing Documentation

== Render Examples

To show before/after formatting examples automatically, use the `render-examples` feature:

+ *Enable automatic rendering* by adding ```typ #show: render-examples``` to your document
+ *Write code examples* using ````typ ```typst ``` ```` code blocks - they will automatically show before/after formatting. You can set the `lang` parameter of `render-examples` to set which raw code language to render as examples (default: ```typ "typst"```)
+ *Configure formatting* using special comments in your examples:

  ````typ
  ```typst
  /// typstyle: wrap_text, max_width=40
  这是一个中文段落，包含链接 https://typst.app/ 和*强调文本*。
  続いて`コード要素`と https://docs.typst.app/ を含む日本語の段落です。
  ```
  ````

*Configuration options* supported in the comment are those available in the embedded typstyle package:
- `max_width=N` - Set line width for this example
- `wrap_text` - Enable text wrapping
- See the embedded typstyle documentation for all available options

The system automatically:
- Parses configuration from `/// typstyle:` comments
- Formats the code with typstyle using the specified config
- Displays side-by-side before/after comparison
- Shows the active configuration options

== Using Callout Components

The documentation supports various callout types for highlighting important information:

#note[This is a standard note callout for general information.]

#important[This is an important callout for critical information that users must be aware of.]

#warning[This is a warning callout for potential issues or dangerous operations.]

#tip[This is a tip callout for helpful suggestions and best practices.]

#caution[This is a caution callout for operations that require careful consideration.]

You can also use custom titles:

#callout(type: "important", title: "Custom Title")[
  This callout has a custom title instead of the default "Important".
]

= Documentation Workflow

+ Write content in Typst format
+ Test locally with ```sh just dev-docs```
+ Build static version with ```sh just build-docs```
+ For GitHub Pages deployment, use ```sh just build-docs-gh``` (sets `--path-to-root /typstyle/` for proper asset paths)
+ Review generated HTML output
