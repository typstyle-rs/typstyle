#import "../book.typ": *

#show: book-page.with(title: "Table and Grid Formatting")

#show: render-examples

= Formatting Rules

typstyle applies intelligent formatting to tables and grids based on content structure:

- General Rules
  - `header`, `footer`, and line comments (`//`) always occupy their own lines.
  - Block comments disable table formatting entirely.
  - Blank lines are preserved and prevent reflow across them.
- Header & Footer
  - Both follow the table’s defined column layout.
- Cell Reflow
  - Reflow applies only when *no special cells* are present.
    Special cells include:
    - `cell`
    - `hline`
    - `vline`
    - Spread args (`..`)
  - If no special cells exist, typstyle reflows all cells to fit the columns.
  - Otherwise, the original grid structure is preserved.

= Column-Aware Formatting

typstyle formats tables and grids in a "column-aware" way, recognizing basic patterns and column numbers. Single rows are kept on single lines when possible:

```typst
#table(
  columns: (auto, 1fr,) + (auto,),
  [Ethanol], [78.2], [241.0],   [Methanol],
   [64.7], [239.5],
  [Propanol], [97.4], [263.7], table.footer(
  repeat: true,     [Alcohols], [Average], [>240°C]
  )
)
```

When a table row cannot fit on a single line, each cell is placed on its own line:

```typst
#figure(
  grid(
    columns: (auto,auto),
    rows: (auto,auto),
    gutter: 0em,
    [#image("assets/1.png",width: 59%)],[#image("assets/2.png", width: 55%)],
    [#image("assets/3.png",width: 1fr)],[#image("assets/4.png", width: 2fr)],

  ),
  caption: [],
)
```

= Advanced Table Features

typstyle provides comprehensive support for complex table structures:

- Headers and footers are formatted as tables
- Special elements (`cell`, `hline`, `vline`) are recognized without prefixes
- Column count calculation handles complex expressions like `((1fr,) * 2 + 2 * (auto,)) * 3`
- Headers, footers, and table cells with rowspan/colspan are properly handled

```typst
/// typstyle: max_width=120
#table(
  columns: 7,
  rows: (2.5em,) * 3,
  align: horizon + center,
  table.cell(rowspan: 2)[],table.cell(colspan: 2)[Header 1],table.cell(colspan: 2)[Header 2],table.cell(colspan: 2)[Header 3],
  ..([Data],[Info]) * 3,
  $-2$,$20degree 9'$,$577.48$,$20degree 17'$,$581.14$,$15degree 5'$,$436.24$,
  $-1$,$9degree 54'$,$576.44$,$9degree 57'$,$579.32$,$7degree 29'$,$436.66$,
)
```
