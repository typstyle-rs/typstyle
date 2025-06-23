
= Advanced Function Arguments

== Flavor Detection

When the first space contains a newline, arguments spread across lines:

#let document-config(
  title,
  authors,
  date: datetime.today()
) = {
  // Configuration logic
}

#let inline-config(title, authors, date: datetime.today()) = {
  // Inline when it fits
}

== Combinable Arguments

Compact layout with combinable last arguments:

#figure(
  caption: [Data visualization],
  placement: top,
  table(
    columns: 3,
    [A], [B], [C],
    [1], [2], [3],
    [4], [5], [6]
  )
)

== Complex Nested Arguments

#let process-pipeline(
  input-data,
  transformations: (
    normalize: true,
    filter-outliers: true,
    fill-missing: "mean"
  ),
  output-format: (
    type: "json",
    compression: "gzip",
    metadata: (
      timestamp: datetime.today(),
      version: "1.0"
    )
  )
) = {
  // Processing logic
}

== Trailing Commas and Comments

#let api-config(
  host: "localhost", // Server address
  port: 8080, /* Default port for development
               Change for production */
  ssl: false, // Enable for production
  timeout: 30, // Connection timeout in seconds
) = {
  // Configuration setup
}

== Mixed Argument Types

#let complex-function(
  required-arg,
  optional: none,
  callback: x => x,
  data: (),
  flags: (verbose: true, debug: false)
) = {
  // Function implementation
}

== Real-world Example

#let create-document(
  content,
  metadata: (
    title: "Document",
    author: "Unknown"
  ),
  styling: (
    font: "Liberation Serif",
    size: 11pt,
    spacing: 1.2
  ),
  options: (
    numbered: true,
    toc: false
  )
) = {
  set text(
    font: styling.font,
    size: styling.size
  )

  if options.toc {
    outline()
    pagebreak()
  }

  content
}
