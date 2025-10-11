#import "./book.typ": *

#show: book-page.with(title: "Formatting Features")

#show: render-examples

Typstyle follows a consistent set of formatting rules to ensure your Typst code is readable, maintainable, and follows established conventions. This page documents the core formatting styles applied by Typstyle.

#callout.note[
  All examples are automatically rendered using the embedded Typstyle formatter, ensuring they always reflect the latest features.

  However, documentation updates may lag behind new features. If there are inconsistencies between descriptions and example output, the actual formatting behavior takes precedence.
]

= Configuration

== Line Width and Indentation

- *Default line width*: 80 characters (configurable with `--line-width`)
- *Default indentation*: 2 spaces per level (configurable with `--indent-width`)
- *File endings*: Documents containing content end with a newline character. Partial formatting follows the selected node's layout.

= Whitespace Preservation

Typstyle preserves trailing spaces and whitespace-only lines inside multiline strings, raw blocks, and source protected by `@typstyle off`. These spaces can affect content and are not removed by a global cleanup pass.

Blank lines created by the layout carry no indentation, and generated flow separators are omitted at line boundaries. The same behavior applies to partial formatting and aligned math.

Comments have their own normalization rules: trailing whitespace is removed from line comments and from block-comment lines with leading stars.

= Comments

== Inline Comments

Inline comments are preserved and positioned correctly:

```typst
#let conf(   title: none,//comments
authors: (),
  abstract: [],
    lang:     "zh",// language
  doctype: "book",//comments

doc,// my docs
) = { doc
}

#{
  let c = 0// my comment
}
```

== Block Comments

Block comments are automatically aligned and formatted:

```typst
#{
  let x = 1   /* Attached block comment
      that spans
 multiple lines
  */

  /* Block comment
      that spans
 multiple lines
  */
}

Aligned: /* Block comment with leading stars
    *  that
        *  spans
 *  multiple
    *  lines
  */
```

= Disabling Formatting

Use `// @typstyle off` to disable formatting for specific code regions:

```typst
// @typstyle off
#let intentionally_bad_format    =    "preserved";
#let properly_formatted="cleaned up"
```

#callout.note[
  The escape hatch only applies to the next syntax node, not the rest of the code.

  There is no closing `// @typstyle on` directive.
]

For details, please see #cross-link("/escape-hatch.typ")[Escape Hatch].
