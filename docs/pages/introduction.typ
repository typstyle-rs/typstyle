#import "./book.typ": *

#show: book-page.with(title: "Introduction")

#context if is-html-target() {
  html.div({
    html.a(href: "https://repology.org/project/typstyle/versions")[
      #html.img(
        src: "https://repology.org/badge/latest-versions/typstyle.svg",
        alt: "latest packaged version(s)",
      )
    ]
  })
}

*Typstyle* is a beautiful and reliable code formatter for #link("https://typst.app/")[Typst].

Typstyle automatically formats your Typst source code to ensure consistency and readability. It's fast, opinionated, and preserves the semantic meaning of your code while improving its appearance.

= Key Features

- *Fast*: Formats large documents (1000+ lines) under 5ms, or a document with 300 huge equations in 15ms.
- *Reliable*: Preserves semantic meaning while improving code appearance.
- *Correct*: The formatted output renders identically to the original source.
- *Opinionated*: Consistent style with minimal configuration.
- *Idempotent*: Formatting the same code multiple times produces identical results — the formatter always converges to a fixed point.

== Correctness & Idempotency Guarantees

Typstyle provides rigorous correctness guarantees verified by automated testing:

- *Idempotency*: Running the formatter twice produces the same output as running it once. The formatted code is always a fixed point of the formatter.
- *Rendering Correctness*: The formatted code renders identically to the original. Typstyle compares rendered outputs (via image diffs) to ensure no visual changes are introduced.
- *Error Consistency*: The same set of warnings and errors are produced before and after formatting. No diagnostic information is lost or altered.

These guarantees are enforced through comprehensive test suites:
- *Snapshot tests*: Every formatting change is validated against reference outputs to prevent regressions.
- *Idempotency tests*: Each test fixture is formatted repeatedly to verify it reaches a stable fixed point.
- *Correctness tests*: Rendered outputs are compared pixel-by-pixel to ensure semantic equivalence before and after formatting.
- *End-to-end tests*: Real-world Typst repositories are formatted and verified against all the above checks.

= Try It Online

Try Typstyle in your browser: #link(package.homepage + "playground")[playground]

The playground integrates the latest version of Typstyle. If you encounter formatting issues, please verify them in the playground first before reporting bugs.

= How It Works

Typstyle parses your code into an Abstract Syntax Tree (AST), applies formatting rules based on Philip Wadler's pretty printing algorithms, and outputs clean, consistently formatted code. For a deeper look at the pipeline and crate structure, see #cross-link("/architecture.typ")[Architecture].
