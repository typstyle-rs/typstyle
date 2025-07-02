
// Helper function for demonstration
#let format-number(num) = str(calc.round(num, digits: 3))

// Function with complex arguments that showcase "flavor detection"
#let render-document(
  title: "Advanced Document",
  authors: ("Alice", "Bob", "Charlie"),
  metadata: (
    version: "2.1",
    tags: ("academic", "research", "typst"),
    config: (
      line-height: 1.5,
      margins: (x: 1in, y: 1.2in)
    )
  ),
  content
) = {
  // Complex nested function calls
  page(
    paper: "a4",
    margin: metadata.config.margins,
    header: align(center)[
      *#title* #h(1fr) Version #metadata.version
    ],
    footer: context [
      #counter(page).display() of #counter(page).final().first()
    ]
  )[
    // Combinable arguments - last argument spans multiple lines
    #figure(
      placement: top,
      caption: [Complex data visualization showcasing Typstyle's formatting],
      supplement: [Chart],
      grid(
        columns: (1fr, 1fr, 1fr),
        rows: (auto, auto),
        stroke: 0.5pt,
        [Data A], [Data B], [Data C],
        [#format-number(123.456)], [#format-number(789.012)], [#format-number(345.678)]
      )
    )

    // Function calls with mixed argument types
    #let process-data(raw-data, filters: (), transform: x => x, options: (:)) = {
      raw-data.filter(filters.fold(true, (acc, f) => acc and f)).map(transform)
    }

    // Complex conditional with nested function calls
    #if authors.len() > 2 {
      align(center)[
        *Authors:* #authors.slice(0, -1).join(", ") and #authors.last()
      ]
    } else if authors.len() == 2 {
      align(center)[
        *Authors:* #authors.join(" and ")
      ]
    } else {
      align(center)[*Author:* #authors.first()]
    }

    content
  ]
}

// Demonstrating parentheses removal and complex expressions
#let data = (
  measurements: "raw-values",
  analysis: (
    statistics: (
      mean: "calculate-mean",
      std-dev: "calculate-std"
    )
  )
)

// Function with trailing commas and comments
#let configure-layout(
  // Basic layout options
  columns: 2,
  gutter: 1em, // Space between columns

  // Advanced options
  balance: true, /* Balance column heights */
  orphans: 2, // Minimum lines at bottom of page
  widows: 2, /* Minimum lines at top of page */

  // Content processing
  preprocess: none, // Optional preprocessing function
  postprocess: none, /* Optional postprocessing function */
) = {
  // Complex layout logic here
  layout(size => {
    let column-width = (size.width - gutter * (columns - 1)) / columns
    // More layout calculations...
  })
}

// Showcasing Typstyle's handling of complex nested structures
#let theme = (
  colors: (
    primary: rgb("#2563eb"),
    secondary: rgb("#7c3aed"),
    accent: rgb("#f59e0b"),
    background: (
      light: rgb("#ffffff"),
      dark: rgb("#1f2937")
    )
  ),
  typography: (
    headings: (
      font: "Libertinus Serif",
      weights: (h1: 700, h2: 600, h3: 500)
    ),
    body: (
      font: "Libertinus Serif",
      size: 11pt,
      leading: 0.65em
    )
  ),
  spacing: (
    sections: 1.5em,
    paragraphs: 0.65em,
    items: 0.35em
  )
)

// @typstyle off - demonstrating escape hatch
#let intentionally_bad_format    =    "This formatting is preserved";
#let properly_formatted = "This will be cleaned up"

// Complex mathematical function with mixed arguments
#let solve-quadratic(a, b, c, precision: 6) = {
  let discriminant = b * b - 4 * a * c
  if discriminant < 0 [
    No real solutions
  ] else if discriminant == 0 [
    Solution: $x = #{calc.round(-b / (2 * a), digits: precision)}$
  ] else [
    Solutions: $x_1 = #{calc.round((-b + calc.sqrt(discriminant)) / (2 * a), digits: precision)}$, $x_2 = #{calc.round((-b - calc.sqrt(discriminant)) / (2 * a), digits: precision)}$
  ]
}

// Example usage with the render-document function
#render-document(
  authors: ("Dr. Jane Smith", "Prof. John Doe"),
  metadata: (
    version: "3.0",
    tags: ("typography", "formatting", "demonstration"),
    config: (
      line-height: 1.6,
      margins: (x: 1.25in, y: 1in)
    )
  )
)[
  = Introduction

  This document demonstrates advanced function call formatting and argument handling in Typstyle.

  #solve-quadratic(1, -5, 6)

  = Data Processing

  #let sample-data = range(10).map(x => (
    id: x,
    value: calc.pow(x, 2) + calc.sin(x),
    category: if calc.rem(x, 2) == 0 { "even" } else { "odd" }
  ))

  #table(
    columns: 3,
    [ID], [Value], [Category],
    ..sample-data.map(item => (
      str(item.id),
      str(calc.round(item.value, digits: 3)),
      item.category
    )).flatten()
  )
]
