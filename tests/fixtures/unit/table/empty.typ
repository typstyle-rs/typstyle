// Empty table
#table()

// Empty table with only columns
#table(columns:    3)

// Empty table with spread args
#table(..(   columns:     2))

// Empty table with multiple named args
#table(   columns:    4,stroke:     blue,     align:   center)

// Table with only header
#table(columns:      2,table.header(repeat:    true))

// Table with only footer
#table(
  columns:   3,table.footer()
)
#table(
  columns:   3,table.footer(

  )
)

// Empty grid with only named args
#grid(columns:    (1fr,     2fr),
  rows:   (auto,     1cm),gutter:      5pt)

// Grid with spread args
#grid(..(columns:    2,
    rows:     auto))

// Mixed named and spread args
#table(stroke:    none,    ..(columns:      3))

// Complex empty table
#table(columns:   (1fr,    auto),align:      center,
  inset:    5pt,stroke:   (thickness:     1pt,
    dash:       "dotted"),
  ..(fill:     rgb(    "eee")))


// No parenthesized args
#table(
  columns: 2,
  table.header[],
)
#table(
  columns: 2,
  table.header[][],
)
