#import "./book.typ": *
#import "../deps.typ": typstyle

#show: book-page.with(title: "Quick Start")

= Quick Start

Get up and running with typstyle in minutes.

== Your First Format

=== Format a Single File

#let example-file = ````typ
// example.typ
#import    "@preview/cetz:0.2.2":  draw,  canvas,
#import"@preview/fletcher:0.5.1"as fletcher:diagram,node,edge
#set   page(  margin:(x:1.2in, y:1in  )  )
#set text(font:("Times New Roman",)
)

= Mathematical Analysis of Algorithm Performance

Consider the time complexity analysis for sorting algorithms.The bubble sort algorithm has quadratic time complexity.

== Performance Comparison

#figure(
table(columns:(auto,  )+(1fr,)*3,
[Algorithm],[Best Case],[Average Case],[Worst Case],
[Bubble Sort],$O(n)$,$O(n^2)$,$O(n^2)$,[Quick Sort],$O(n log n)$,$O(n log n)$,$O(n^2)$,
[Merge Sort],$O(n log n)$,$O(n log n)$,$O(n log n)$),caption:[Time complexity comparison]
)

The mathematical relationship between input size and execution time can be expressed as:

$
T(n)&=sum_(i=1)^(n-1) sum_(j=1)^(n-i) 1\
&= sum_(i=1)^(n-1) (n-i)\
&= sum_(k=1)^(n-1) k   "where" k=n-i\
&= (n-1)n/2\
&= O(n^2)
$

#figure(canvas(length:1cm,{
draw.plot.plot(size:(8,6),x-tick-step:none,y-tick-step:none,{
draw.plot.add(((0,0),(1,1),(2,4),(3,9),(4,16)))
})}),caption:[Quadratic growth visualization])
````

Create a simple Typst file:

#example-file

Format it with typstyle:

```bash
typstyle example.typ
```

Output:

#raw(typstyle.format(example-file.text), lang: "typ", block: true)

=== Format In-Place

To modify the file directly:

```bash
typstyle -i example.typ
```

=== Format from stdin

```bash
cat example.typ | typstyle > formatted.typ
```

== Common Usage Patterns

=== Format Multiple Files

```bash
# Format specific files
typstyle -i file1.typ file2.typ

# Format entire directory
typstyle -i src/

# Format with specific line width
typstyle -l 100 -i src/
```

=== Check Mode

Use check mode in CI/CD to ensure code is properly formatted:

```bash
typstyle --check src/
typstyle --diff src/
```

This exits with code 0 if files are properly formatted, non-zero otherwise.

== Configuration Options

=== Line Width

```bash
# Set maximum line width to 100 characters
typstyle -l 100 file.typ
```

=== Indentation

```bash
# Use 4 spaces for indentation instead of default 2
typstyle -t 4 file.typ
```

=== Import Sorting

```bash
# Disable automatic import sorting
typstyle --no-reorder-import-items file.typ
```

=== Text Wrapping

```bash
# Enable text wrapping in markup
typstyle --wrap-text file.typ
```

== Integration Examples

=== VS Code

1. Install Tinymist extension
2. Add to `settings.json`:
  ```json
  {
    "tinymist.formatterMode": "typstyle",
    "editor.formatOnSave": true
  }
  ```

== Next Steps

- Learn about #cross-link("/features.typ")[features]
- See #cross-link("/cli-usage.typ")[detailed CLI usage]
