#import "./book.typ": *

#show: book-page.with(title: "Formatting Features")

#show: render-examples

Typstyle follows a consistent set of formatting rules to ensure your Typst code is readable, maintainable, and follows established conventions. This page documents the core formatting styles applied by typstyle.

#callout.note[
  All examples are automatically rendered using the embedded typstyle formatter, ensuring they always reflect the latest features.

  However, documentation updates may lag behind new features. If there are inconsistencies between descriptions and example output, the actual formatting behavior takes precedence.
]

= Configuration

== Line Width and Indentation

- *Default line width*: 80 characters (configurable with `--line-width`)
- *Default indentation*: 2 spaces per level (configurable with `--indent-width`)
- *File endings*: typstyle ensures files end with a newline character

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
