#import "./book.typ": *

#show: book-page.with(title: "Installation")

= Installation & Setup

Typstyle can be installed and used in multiple ways. Choose the method that best fits your workflow.

== CLI Installation

=== Download Binary

The easiest way to get started is to download the pre-built binary from the #link("https://github.com/typstyle-rs/typstyle/releases")[release page].

=== Package Managers

#context if is-html-target() {
  html.elem("a", attrs: (href: "https://repology.org/project/typstyle/versions"))[
    #html.elem("img", attrs: (
      src: "https://repology.org/badge/vertical-allrepos/typstyle.svg",
      alt: "Packaging status",
      align: "right",
    ))
  ]
}

Typstyle is available in many package managers. Check the #link("https://repology.org/project/typstyle/versions")[packaging status] for your distribution.

Notably, typstyle is available in #link("https://www.archlinuxcn.org/archlinux-cn-repo-and-mirror/")[Archlinux CN] repo.

=== Cargo Installation

==== Using cargo-binstall (Recommended)

```bash
cargo binstall typstyle
```

==== Building from Source

```bash
cargo install typstyle --locked
```

== Editor Integration

Typstyle has been integrated into #link("https://github.com/Myriad-Dreamin/tinymist")[tinymist]. You can use it in your editor by installing the tinymist plugin and set `tinymist.formatterMode` to `typstyle`.

=== VS Code (via Tinymist)

+ Install the #link("https://marketplace.visualstudio.com/items?itemName=myriad-dreamin.tinymist")[Tinymist extension]
+ Set `tinymist.formatterMode` to `"typstyle"` in your settings
+ Enable format on save or use `Ctrl+Shift+P` → "Format Document"

== Library Installation

Typstyle is also available as a library integrated in your project.

=== Cargo (Rust)

#let ver = toml("../../Cargo.toml").workspace.package.version

#raw("[dependencies]
typstyle-core = \"=" + ver + "\"", lang: "toml", block: true)

*Note*: Typstyle follows Typst’s major and minor versioning, and even patch releases may introduce breaking changes. We recommend pinning the version in your dependency and upgrading only when you require new features.

=== NPM (JavaScript/TypeScript)

For web projects using WebAssembly bindings:

#raw("npm install @typstyle/typstyle-wasm-bundler", lang: "bash", block: true)

The `@typstyle/typstyle-wasm-bundler` package provides WebAssembly bindings optimized for web bundlers like Webpack, Vite, and Rollup.

== GitHub Actions

Use the 3rd party-maintained #link("https://github.com/grayespinoza/typstyle-action")[typstyle-action] by #link("https://github.com/grayespinoza")[grayespinoza]:

```yaml
- name: Run typstyle
  uses: grayespinoza/typstyle-action@main
```

== Pre-commit Hook

Not properly done yet.
