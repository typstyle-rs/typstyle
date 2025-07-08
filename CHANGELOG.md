# Changelog

## Unreleased

We now employ [prettyless](https://github.com/typstyle-rs/prettyless) as the layout engine. It provides more features to support us to adopt better layout logic.

- Feature: Long lines after the first line will no longer cause compact layout to fail to fit.
  For example:
  ```typst
  #figure(caption: [], {
    "looooooooooooooooooooooooooong"
  })
  ```
  It now fits in the compact layout regardless of the length of any long line in the code block.

- Feature: We now respect the flavor of arguments in any case, except that the sole argument is a code block, which is always folded. This reverts the behavior of "force folding" when the sole argument is combinable, so that users can freely decide whether the argument should be compact. This change is expected not to affect your code if it is already compact.
  For example:
  ```typst
  #f(
    (1, 2, 3)
  )
  ```
  was formatted to
  ```typst
  #f((1, 2, 3))
  ```
  Now it is kept unchanged given its multiline flavor.

- Bug fix: In code mode, `std.table` was not recognized as a table function, preventing proper table formatting.

- Bug fix: Fixed incorrect formatting of table functions with empty content blocks.
  When `table.header` or `table.footer` had no parentheses (`()`), the formatter would incorrectly add parentheses before empty content blocks. When parenthesized args are empty, the formatter would insert a line break with short line length.
  For example:
  ```typst
  #table(
    columns: 2,
    table.header[],
    table.footer(),
  )
  ```
  was formatted to:
  ```typst
  #table(
    columns: 2,
    table.header()[],
    table.footer(),
  )
  ```
  or with short line length:
  ```typst
  #table(
    columns: 2,
    table.header(

    )[],
    table.footer(

    ),
  )
  ```

- Bug fix: Fixed incorrect handling of trailing content blocks in set rules.
  Previously, when there were no parentheses, a following content block would be wrapped by `()`; when parentheses were already present, any trailing content block was dropped.
  For example:
  ```typst
  #set circle[1]
  #set circle()[1]
  #set circle(stroke: blue)[1]
  ```
  was previously formatted to:
  ```typst
  #set circle([1])
  #set circle()
  #set circle(stroke: blue)
  ```

## v0.13.13 - [2025-07-02]

- Bug fix(CI): Fixed a CI misconfiguration that prevented the release of v0.13.12.

## v0.13.12 - [2025-07-02]

🎉 typstyle has moved to https://github.com/typstyle-rs/typstyle !

- Bug fix(https://github.com/typstyle-rs/typstyle/issues/334): Fixed incorrect formatting when term description is empty in definition lists.

- Feature: Improved argument layout logic - do not use compact layout when arguments have multiline flavor, providing better formatting for complex function calls.

- Feature: Enhanced math formatting - no longer align backslashes in mathematical expressions for cleaner output.

- Feature: Introduced `typstyle-typlugin` crate for embedding typstyle as a WebAssembly plugin within Typst documents.

## v0.13.11 - [2025-06-14]

- Feature(CLI): (breaking) Removed deprecated `format-all` command from CLI. This command was previously deprecated and has now been completely removed. Users should use `typstyle <dir> -i` instead.

- Feature(CLI): Added `--timing` option to CLI for displaying elapsed time during formatting operations. Users can now monitor the performance of formatting operations with this new debug flag.

- Feature: Empty code blocks no longer get extra spaces added.
  For example, `#{}` will now be formatted as `#{}` instead of `#{ }`. This aligns with the behavior of most code formatters.

- Enhancement: Labels are no longer reflowed to the next line with text wrapping enabled.

- Bug fix: Texts spanning multiple lines, such as strings, inline raws, or syntax nodes marked `typstyle off` are correctly layouted according to their line widths.

- Bug fix: When the last arg of func call is not combinable, the flavor is respect.

## v0.13.10 - [2025-06-05]

- Bug fix: Fixed unexpected semicolon to comma conversion in nested math arguments. Previously, typstyle would incorrectly convert semicolons to commas in complex nested mathematical expressions like `$mat(mat(1; 2); mat(3; 4))$`.

- Refactor: Extracted WASM features into a separate `typstyle-wasm` crate. This improves the modular architecture by separating WebAssembly-specific functionality from the core library.

- API Enhancement: Added rendering IR functionality to the formatter. The `Formatter` now provides a `render_ir()` method that allows developers to inspect the pretty-printing intermediate representation, which can be useful for debugging and understanding the formatting process.

## v0.13.9 - [2025-05-25]

- Feature: typstyle now evaluates simple constant expressions for table columns. This enhancement allows for better column count calculation in tables. For example, if you have a table with a column count defined as `((1fr,) * 2 + 2 * (auto,)) * 3`, typstyle will now correctly interpret this as 12 columns.

- Feature: typstyle now uses soft wrapping for import items. Import statements with long lists of items will now wrap more compact. This aligns with rustfmt's approach to formatting long import lists.

- Feature: typstyle no longer collapses consecutive spaces in markup by default. This change preserves intentional spacing in markup content, maintaining the author's formatting intentions.

- Feature: typstyle no longer adds padding to the last cell in math alignments. For example, in `cases()` expressions, the formatter will no longer add trailing spaces after the last cell, keeping comments properly aligned without unnecessary padding.

- Enhancement: typstyle now formats tables in most cases, and format headers and footers as tables. This major enhancement provides better support for complex table structures.

- Enhancement: improved detection of table and grid elements. Special rows (`header`, `footer`) and cells (`cell`, `hline`, `vline`) are now recognized without needing the `grid.` or `table.` prefix.

- Bug fix: typstyle now preserves backslashes in single-row aligned math equations. This addresses a missed case from #294 where expressions like `math.equation($ 1 & 2 \ $.body + $ & 4 \ $.body)` would incorrectly have their backslashes removed.

- Bug fix: typstyle now handles escape hatch comments (`@typstyle off`) better in corner cases. Previously, escape hatch did not work for `ArrayItem`, `DictItem`, `Param`, and `DestructuringItem`. Additionally, `@typstyle off` no longer penetrates comments, and when it appears before `Code` or `Math`, it now only applies to the first non-trivial child instead of the whole syntax node.

## v0.13.8 - [2025-05-21]

- Bug fix: typstyle previously will break inline equations if they has alignments. Now it is fixed. It will never add paddings to align inline equations now.
- Feature: typstyle doesn't enforce trailing backslash in math equations with alignments now.
- Feature: Previously, typstyle will try to put the last argument of a function call in the same line as the function call when certain conditions are met. Now it is enhanced. In this version, if the last argument is an array or a dict, typstyle will only do this if it is the only array/dict argument.

## v0.13.7 - [2025-05-15]

- Cli: typstyle now supports formatting dirs natively. You can use `typstyle -i <dir>` to format all files in the directory. Given that, `typstyle format-all` is deprecated and will be removed in the future.
- Cli: typstyle now uses `-l` or `--line-width` instead of `-c`/`--column` to specify the line width. The `-c`/`--column` option is deprecated and will be removed in the future.
- Cli: typstyle now uses `--indent-width` instead of `--tab-width` to specify the indent width. The `--tab-width` option is deprecated and will be removed in the future. However, `-t` is still available.
- Feature: when `--wrap-text` is enabled, typstyle will now keep the line breaks after backslashes in markup. Previously, it is not treated specially.
- Feature: typstyle now keep spaces around fractions in math equations. Previously, it always added a space before and after the fraction.

## v0.13.6 - [2025-05-11]

- Bug fix: #272. Previously, typstyle will remove the space between variable and underscore in math equations. `$ #mysum _(i=0) $` for example, will be formatted as `$ #mysum_(i=0) $`. Now it is fixed.
- Enhancement: #273. When formatting math equations with alignments and multiline cells, previous typstyle versions will introduce excessive spaces. Now it is fixed.
- Enhancement: #280. When wrap text is enabled, previous typstyle will mess up the formatting of labels. Now it is fixed.

## v0.13.5 - [2025-05-07]

- Bug fix: Typstyle previously will panic if there is only a space in math function call. Now it is fixed.
- Bug fix: Typstyle previously will not converge if there are multiple newlines in a function call. Now it is fixed.

## v0.13.4 - [2025-04-30]

- Feature: typstyle cli now enables import sorting by default. You can disable it with `--no-reorder-import-items` flag.

- Feature: typstyle now supports `--wrap-text` flag to wrap texts in the markup. It is considered as experimental and may not work in all cases. Please report any issues you encounter.

  For example, this code:
  ```typst
  Let's say you have a long text that needs to be wrapped in the markup. This is a very long sentence that needs to be wrapped in the markup. It should be wrapped in the markup.
  ```

  Will be formatted as following when column width is 80 and `--wrap-text` is enabled:
  ```typst
  Let's say you have a long text that needs to be wrapped in the markup. This is a
  very long sentence that needs to be wrapped in the markup. It should be wrapped
  in the markup.
  ```

- Feature: typstyle now tries to align `&`s in math equations, even if the cells are multiline. Currently it it works in most cases, except the following:
  - Has multiline `Str` or `Raw` descendants.
  - Not following a linebreak when in `MathDelimited` or `Args`.
  - Across args of functions such as `cases`.

  For example, this code:
  ```typst
  $
  F_n &= sum_(i=1)^n i^2 & n > 0 \
  a &< b+1 & forall b < 1
  $
  ```
  Will be formatted as:
  ```typst
  $
    F_n & = sum_(i=1)^n i^2 &        n > 0 \
      a & < b+1             & forall b < 1 \
  $
  ```

- Feature: typstyle now generate more compact result when formatting complex function calls. Similar to what [rustfmt does](https://doc.rust-lang.org/nightly/style-guide/expressions.html#combinable-expressions), when the only argument is combinable, it will be put in the same line as the function call. When the last argument is blocky, it will also be put in the same line if possible.

  For example, this code:
  ```typst
  #figure(
    fletcher.diagram(
      node-outset: .5em,
      node-stroke: .075em,

      node(
        (+1,0,),
        [variable],
        radius: 3em,
      ), // test
    ))
  ```

  Will be formatted as:
  ```typst
  #figure(fletcher.diagram(
    node-outset: .5em,
    node-stroke: .075em,

    node((+1, 0), [variable], radius: 3em), // test
  ))
  ```

  For another example, this code:
  ```typst
  #set page(
    margin: 0.5in,
    footer: context {
      if counter(page).display() == "2" {
        [test]
      } else {
        []
      }
    }
  )
  ```

  Will be formatted as:
  ```typst
  #set page(margin: 0.5in, footer: context {
    if counter(page).display() == "2" {
      [test]
    } else {
      []
    }
  })
  ```

  Due to the limitations of the currently used pretty engine, there is still room for improvement in some cases.

## v0.13.3 - [2025-04-10]

- Feature: Unified equation layout and removed indent for non-block equations
  - Non-block equations are no longer indented, which works well in most reasonable cases
  - Nodes with multiline comments are now treated as multiline
  - Improved comment attachment mechanism
- Fix: Improved handling of comments directly in equations and spaces in math
  - Correctly handles in-equation comments before and after `Math`
  - Fixed infinite spaces added before `//` in math
  - Fixed extra indent of array args in math
  - Optimized spaces handling in `MathAttach` and `MathRoot`
- Fix: Addressed list layout issues
  - Fixed indent and grouping issues in list layouts
  - Fixed missing separators in list layouts mixed with comments

## v0.13.2 - [2025-03-28]

- Feature: Typstyle can format code in markup lines now. Linebreaks are suppressed inside to ensure compact layout.
- Feature: Typstyle can format math equations with comments and hashes now. Previously they are skipped.
- Feature: Improved formatting of math delimited. Linebreaks are kept as is.
- Experimental feature: Typstyle CLI will sort import items in a single import statement if `--reorder-import-items` is passed. It would be enabled by default in the future.
- Added a 3rd-party [typstyle-action](https://github.com/grayespinoza/typstyle-action) maintained by [@grayespinoza](https://github.com/grayespinoza).

## v0.13.1 - [2025-03-20]

- Bump to typst v0.13.1
- Typstyle now uses braces ({}) instead of parens (()) when the closure body is not a chainable binary expression

```typst
#fun(() => {
  aaa
})
instead of
#fun(() => (
  aaa
))
```
- Fix mis-format when linebreak `\` appears at the end of a inline math equation.
- Fix mis-format when line comment appears in a multiline `import` statement.

## v0.13.0 - [2025-02-22]

- Bump to typst v0.13.0
- Regression: In typst v0.13.0, PR [#5310](https://github.com/typst/typst/pull/5310) changes the parsing behavior of comments when it presenting in list, enum and term list. In this verison, the indent level of the comment in list, enum and term list determined by the **next** item, not the previous item. For example,
```typst
- Fruit
  - Apple
  - Banana
  // - Orange
- Vegetable
  - Carrot
  - Tomato
```

will be formatted as this in this version. Note that the `// - Orange` is misindented. It is indented to the same level as `- Vegetable`. It works perfectly in previous versions. So if you want to keep the old behavior, please keep using typstyle v0.12.x at this moment.
```typst
- Fruit
  - Apple
  - Banana
// - Orange
- Vegetable
  - Carrot
  - Tomato
```

## v0.12.15 - [2025-02-16]

- Feat: add `--tab-width` cli option to set the number of spaces for indentation. The default value is 2.
- Fix: typstyle-cli now outputs the original source when the input syntax is erroneous.
- Fix: issues with list/enum/term item indent and linebreak with comments are fixed. Now linebreaks in items are preserved. Items in content blocks will be surrounded with linebreaks when necessary (also to avoid ambiguity).

For example, the code

```typst
+
  + xyz

-
  xyz

- //foo
  - xyz
  //bar

/ 4:
  // 4
  / 44: // 44
    444
```

was incorrectly formatted to

```typst
+ + xyz

- xyz

- //foo
- xyz
  //bar

/ 4: // 4
  / 44: // 44
  444
```

Now it is correctly unchanged.

And

```typst
#{
  [- single]
  [- indented
  - less
  ]
  [- indented
   - same
  - then less
   - then same
  ]
  [- indented
    - more
   - then same
  - then less
  ]
}
```

will be correctly formatted to

```typst
#{
  [- single]
  [
    - indented
    - less
  ]
  [
    - indented
    - same
    - then less
      - then same
  ]
  [
    - indented
      - more
    - then same
    - then less
  ]
}
```

## v0.12.14 - [2024-12-27]

- Fix: typstyle-cli previously add an extra newline at the end of the file. Now it is fixed.
- Fix: typstyle now correctly strip leading spaces in markup
- Feat: typstyle-core now has very basic support for range formatting.

## v0.12.13 - [2024-12-21]

- Fix: typstyle previously incorrectly remove comments in math equations. Now it is fixed.
- Cli:
  - typstyle now reports error when the input file is invalid.
  - `typstyle --check` no longer changes the file content.
  - Other minor improvements.

## v0.12.12 - [2024-12-16]

No changes. We failed to publish the previous release because of the ci issue.

## v0.12.11 - [2024-12-16]

**Packaging:** We've split typstyle crate into two independent crates: `typstyle` and `typstyle-core`. `typstyle` is the CLI tool, and `typstyle-core` is the core library. The npm package is now `typstyle-core`, `typstyle` on npm will be deprecated in the future.

- Fix: `// @typstyle off` not working in certain cases. See #182
- Feature: dot chain formatting is more smart now. We will only break dot chain into multiple lines if it is long enough or complex enough. For example, the following result is generated by typstyle previously:
```typst
#{
  cetz
    .draw
    .group({
      cetz.draw.translate(node.pos.xyz)
      for (i, extrude) in node.extrude.enumerate() {
        cetz
          .draw
          .set-style(
            fill: if i == 0 { node.fill },
            stroke: node.stroke,
          )
        (node.shape)(node, extrude)
      }
    })
}
```

Now it will be formatted as following. This is more readable and compact:
```typst
#{
  cetz.draw.group({
    cetz.draw.translate(node.pos.xyz)
    for (i, extrude) in node.extrude.enumerate() {
      cetz.draw.set-style(
        fill: if i == 0 { node.fill },
        stroke: node.stroke,
      )
      (node.shape)(node, extrude)
    }
  })
}
```

## v0.12.10 - [2024-12-12]

- Fix: musl build is now statically linked. This fixes the issue that the musl build doesn't work on systems other than alpine.
- Typstyle now break content blocks into multiple lines if they have leading spaces and trailing spaces.

For example, the following code is not formattable by typstyle previously:
```typst
#{
  let res = if true [ The Result is definitely true. And it is a very long sentence that needs to be broken into multiple lines. ] else [ The Result is definitely false. And it is a very long sentence that needs to be broken into multiple lines. ]
}
```

Now it will be formatted as:
```typst
#{
  let res = if true [
    The Result is definitely true. And it is a very long sentence that needs to be broken into multiple lines.
  ] else [
    The Result is definitely false. And it is a very long sentence that needs to be broken into multiple lines.
  ]
}
```

## v0.12.9 - [2024-12-08]

- Typstyle no longer force single arg function call to be in a single line. Instead, it is determined in a smarter way. And this fixes https://github.com/Enter-tainer/typstyle/issues/143.
- Typstyle will always add newline at the end of the file. Previously, it only adds newline when it already exists.

## v0.12.8 - [2024-12-07]

- Typstyle will format binary expressions as operator chains. Parentheses are added if necessary.
- Formatting chains with comments is supported now. This is the last piece of formatting with comments.
- Dot chains in markup with parentheses will be broken into multiple lines, if the it contains at least two dots and one function calls.

For example, [the following code](https://github.com/flaribbit/indenta/blob/a6a0c3fa45b4f16f1944b7078678fa4011bbc1fb/lib.typ#L4C1-L5C1):
```typst
#let _is_block(e,fn)=fn==heading or (fn==math.equation and e.block) or (fn==raw and e.has("block") and e.block) or fn==figure or fn==block or fn==list.item or fn==enum.item or fn==table or fn==grid or fn==align or (fn==quote and e.has("block") and e.block)
```

Will be formatted as this in previous versions:
```typst
#let _is_block(e, fn) = (
  fn == heading or (fn == math.equation and e.block) or (
    fn == raw and e.has("block") and e.block
  ) or fn == figure or fn == block or fn == list.item or fn == enum.item or fn == table or fn == grid or fn == align or (
    fn == quote and e.has("block") and e.block
  )
)
```

Now it will be formatted as:
```typst
#let _is_block(e, fn) = (
  fn == heading
    or (fn == math.equation and e.block)
    or (fn == raw and e.has("block") and e.block)
    or fn == figure
    or fn == block
    or fn == list.item
    or fn == enum.item
    or fn == table
    or fn == grid
    or fn == align
    or (fn == quote and e.has("block") and e.block)
)
```


## v0.12.7 - [2024-12-04]

- Dot chain related improvement:
  - Previously if the last item of a dot chain is a function call, typstyle doesn't indent it correctly. Now it is fixed.
  - Previously typstyle formats function calls in dot chains in a very conversative way. Now it is the same as normal function calls.
- Function calls with comments are made formattable.

For example, the following code is not formattable by typstyle previously:
```typst
#{
  let x = f(
  cetz.draw.super-long-name.line(
    start: (0, 0),
    end: (1, 1),      // note
  ) // my comment
)
}
```

Now it will be formatted as:
```typst
#{
  let x = f(
    cetz
      .draw
      .super-long-name
      .line(
        start: (0, 0),
        end: (1, 1), // note
      ), // my comment
  )
}
```

## v0.12.6 - [2024-12-02]

- Parenthesized expressions with comments can be formatted by typstyle now.
- Closure with comments can be formatted by typstyle now.
- Typstyle will removes unnecessary parentheses if the inner expression is literal, array, dict, destructuring, block, or pattern. For safety, parens around idents are kept.
- Destructuring and params with comments are no longer forced to fold into one line.

## v0.12.5 - [2024-11-29]

- Typstyle can format comments appears in most places. Previously it simply gives up when it encounters comments in these places. Now it can format them correctly.

For example, this code:
```typst
#let conf(
  title: none,   //comments
  authors:      (),
  abstract: [],
  lang: "zh",   // language
  doctype: "book",  //comments
  doc  // my docs
) = {
    doc }
```

Previously typstyle will not format it. Now it will be formatted as:
```typst
#let conf(
  title: none, //comments
  authors: (),
  abstract: [],
  lang: "zh", // language
  doctype: "book", //comments
  doc, // my docs
) = {
  doc
}
```

However, there are still some limitations. For more information, see [Limitation](https://enter-tainer.github.io/typstyle/limitations/#expressions-with-comments).

- Fix typstyle previously would format parenthesized patterns incorrectly into `none`. Now it is fixed.

## v0.12.4 - [2024-11-26]

- Performance improvement(#158, #159 by @QuadnucYard): Typstyle now becomes 10-100x faster than before. Previously formatting tablex source code takes ~500ms, but now it only takes less than 5ms.

## v0.12.3 - [2024-11-24]

- Fix doc test failure that prevents nixpkgs from building typstyle.

## v0.12.2 - [2024-11-23]

Introducing new contributor: @QuadnucYard. Welcome! 🎉

- For single item code block, typstyle will try to keep it inline if it fits in a single line and it's inline in original code.

For example, you will get following code:
```typst
#{
  let x = if true { 1 } else { 2 }
}
```

Instead of:
```typst
#{
  let x = if true {
    1
  } else {
    2
  }
}
```

- Typstyle now strip excessive newlines in code blocks. Previously, typstyle will keep all newlines in code blocks. Now it will strip newlines at beginning and end of code blocks. It will also strip newlines in the middle of code blocks if there are more than 2 consecutive newlines.

For example, the following code:
```typst
#{


  let x = 1

  let y = 2

}
```

After formatting, it will become:
```typst
#{
  let x = 1

  let y = 2
}
```

- Formatting block comments are now improved. Previously, leading spaces in block comments are blindly removed. Now typstyle will keep leading spaces in block comments if they are consistent. Typstyle will also try to align `*` in block comments.

For example, the following code:
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

  /* Block comment with leading stars
    *  that
        *  spans
 *  multiple
    *  lines
  */
}

```

Will be formatted as:
```typst
#{
  let x = 1 /* Attached block comment
                 that spans
            multiple lines
             */

  /* Block comment
       that spans
  multiple lines
   */

  /* Block comment with leading stars
   *  that
   *  spans
   *  multiple
   *  lines
   */
}
```


- Fix: `context` expressions are now longer surrounded by unneeded parentheses. Previously, if a context expression spans multiple lines, typstyle will wrap it in parentheses. Now it doesn't.

## v0.12.1 - [2024-11-03]

- Typstyle now keeps spaces around math-delimited when there is already space around it. This prevents a bug when removing the space can cause wrong format result.

For example, this code:
```typst
$[ | | ]$
```

Previous:

```typst
$[| |]$
```

Now it is fixed.

## v0.12.0 - [2024-10-19]

- Bump to typst v0.12.0
- Support new import syntax. Now long import can be broken into multiple lines.

Previous:

```typst
#import "test.typ": aaa, bbb as cccccccccc, ddd as eeeeeeeeeee, fff as g
```

Now:

```typst
#import "test.typ": (
  aaa,
  bbb as cccccccccc,
  ddd as eeeeeeeeeee,
  fff as g,
)
```

## v0.11.35 - [2024-10-07]

- Fix block comments drifting right if they have indentation. Now we strips all leading whitespaces in block comments.

## v0.11.34 - [2024-09-22]

- Fix a bug in the `completions` subcommand. https://github.com/Enter-tainer/typstyle/pull/131#issuecomment-2365456088

## v0.11.33 - [2024-09-22]

- feat: add command-line completions
```
Generate shell completions for the given shell to stdout

Usage: typstyle completions <SHELL>

Arguments:
  <SHELL>  The shell to generate completions for [possible values: bash, elvish, fish, powershell, zsh]
```

## v0.11.32 - [2024-08-19]

- Bug fix: Typstyle previously fails to correctly format inline triple backtick code block without a lang tag or an empty inline triple backtick code block with only a lang tag. Now it is fixed.
```typst
#text(``` test ```)
#text(```test ```)
```

Previously, it will be formatted as:
```typst
#text(```test  ```)
#text(```test  ```)
```

Now it is fixed.

## v0.11.31 - [2024-08-08]

- Bug fix: Typstyle previously removes necessary leading colon in dict. Now it is fixed.
```typst
#{
  let a = (a: 1)
  let b = (b: 2)
  (: ..a, ..b) // previously it will be formatted as (..a, ..b)
}
```

## v0.11.30 - [2024-07-14]

- Bug fix: previously when a destructing pattern has extra parentheses, typstyle will completely remove everything inside the parentheses. Now it is fixed.
- Typstyle now collapses extra parentheses in expression.

## v0.11.29 - [2024-07-13]

- typstyle cli now can be installed from `cargo-binstall`
- typstyle now recognize dot chains and keep them aligned on multiple lines when possible.

Previously, typstyle's format result looks like this:
```typst
#{
  let (title, _) = query(heading.where(level: 1)).map(e => (
    e.body,
    e.location().page(),
  )).rev().find(((_, v)) => v <= page)
}

```

Now it will be formatted as:
```typst
#{
  let (title, _) = query(heading.where(level: 1))
    .map(e => (e.body, e.location().page()))
    .rev()
    .find(((_, v)) => v <= page)
}
```

- Minor adjustment for closure body formatting.

## v0.11.28 - [2024-06-25]

- typstyle cli now has a `--check` flag to check if the input is formatted. If it's not formatted, it will return a non-zero exit code.
- Allow disabling git info collection in build time.

## v0.11.27 - [2024-06-20]

- Fix #97. Typstyle previously add an extra newline for `table` and `grid` when there is no positional argument and there are extra arguments. Now it doesn't add an extra newline.
- Typstyle cli now returns non-zero exit code when there are formatting errors.

## v0.11.26 - [2024-06-13]

- Typstyle now keeps newlines in function call args. Multiple newlines in function call args are common in fletcher diagrams. Before this release, typstyle removes all extra newlines in function call args. Now it keeps them as they are.

<details><summary>Example</summary>

```typst
#set text(10pt)
#diagram(
  node-stroke: .1em,
  node-fill: gradient.radial(blue.lighten(80%), blue, center: (30%, 20%), radius: 80%),
  spacing: 4em,

  node((0,0), `reading`, radius: 2em),
  node((1,0), `eof`, radius: 2em),
  node((2,0), `closed`, radius: 2em, extrude: (-2.5, 0)),

  edge((-1,0), "r", "-|>", `open(path)`, label-pos: 0, label-side: center),
  edge(`read()`, "-|>"),
  edge(`close()`, "-|>"),
  edge((0,0), (0,0), `read()`, "--|>", bend: 130deg),
  edge((0,0), (2,0), `close()`, "-|>", bend: -40deg),
)

```

After formatting, it will become this. Notice the extra newlines are kept.
```typst
#set text(10pt)
#diagram(
  node-stroke: .1em,
  node-fill: gradient.radial(
    blue.lighten(80%),
    blue,
    center: (30%, 20%),
    radius: 80%,
  ),
  spacing: 4em,

  node((0, 0), `reading`, radius: 2em),
  node((1, 0), `eof`, radius: 2em),
  node((2, 0), `closed`, radius: 2em, extrude: (-2.5, 0)),

  edge((-1, 0), "r", "-|>", `open(path)`, label-pos: 0, label-side: center),
  edge(`read()`, "-|>"),
  edge(`close()`, "-|>"),
  edge((0, 0), (0, 0), `read()`, "--|>", bend: 130deg),
  edge((0, 0), (2, 0), `close()`, "-|>", bend: -40deg),
)
```
</details>


- For tables, if typstyle is [unable to format it in a column-aware way](https://enter-tainer.github.io/typstyle/limitations/#table), it will now format each arg, but do not reflow them. That is, the relative position of each arg is kept. If you put something in a single line, it will stay in a single line. Newlines are also kept.

<details><summary>Example</summary>

```typst
#table(
  columns: 4 * (1fr,),

  [a], [b], [c], [d],
  fill: (_, y) => if y == 0 { black },
  table.cell(rowspan: 2)[aa], table.cell(colspan: 2)[bc], [d],
  [b], table.cell(colspan: 2)[cd],
)
```

After formatting, it will become this. Notice the relative position of each arg is kept.
```typst
#table(
  columns: 4 * (1fr,),

  [a], [b], [c], [d],
  fill: (_, y) => if y == 0 {
    black
  },
  table.cell(rowspan: 2)[aa], table.cell(colspan: 2)[bc], [d],
  [b], table.cell(colspan: 2)[cd],
)
```

</details>

## v0.11.25 - [2024-06-09]

- Typstyle now keeps extra newlines in markup mode. Multiple newlines are sometimes used to separate different sections in a document or act as a paragraph placeholder. Typstyle now keeps them as they are.

```typst
== Unfinished Title



=== Section 1



=== Section 2
```

Previously, it will be formatted as:
```typst
== Unfinished Title

== Section 1

== Section 2
```

Now it is kept as it is.

## v0.11.24 - [2024-05-27]

- Now typstyle can format table with `table.header` and `table.footer` attributes. The header and footer will be put in a single line if possible.
  For what it cannot do, see https://github.com/Enter-tainer/typstyle/issues/59#issuecomment-2132252514.

```typst
#table(
  columns: 3,
  table.header(
    [Substance],
    [Subcritical °C],
    [Supercritical °C],
    repeat: true,
  ),
  [Hydrochloric Acid],
  [12.0],
      [92.1],
  [Sodium Myreth Sulfate],
  [16.6],
  [104],
        [Potassium Hydroxide],
  [24.7],
  [114.514],
)
```

After formatting, it will become:
```typst
#table(
  columns: 3,
  table.header(
    [Substance],
    [Subcritical °C],
    [Supercritical °C],
    repeat: true,
  ),

  [Hydrochloric Acid], [12.0], [92.1],
  [Sodium Myreth Sulfate], [16.6], [104],
  [Potassium Hydroxide], [24.7], [114.514],
)
```

## v0.11.23 - [2024-05-25]

- Enhance table formatting. When a table row cannot fit in a single line, each cell will be put in a single line.

For example, this code:
```typst
#figure(
  grid(
    columns: (auto, auto),
    rows: (auto, auto),
    gutter: 0em,
    [ #image("assets/1.png", width: 59%) ], [ #image("assets/2.png",width: 55%) ],

  ),
  caption: [],
)
```

After formatting, it will become:
```typst
#figure(
  grid(
    columns: (auto, auto),
    rows: (auto, auto),
    gutter: 0em,
    [ #image("assets/1.png", width: 59%) ],
    [ #image("assets/2.png", width: 55%) ],
  ),
  caption: [],
)
```
## v0.11.22 - [2024-05-20]

- Typstyle now can format table and grid in a "column-aware" way. It now recognizes basic patterns and column numbers, and put a single row in a single line if possible.

For example, this code:
```typst
#table(
  columns: 3,
    [Substance],
    [Subcritical °C],
    [Supercritical °C],

  [Hydrochloric Acid],
  [12.0], [92.1],
  [Sodium Myreth Sulfate],
  [16.6], [104],
  [Potassium Hydroxide],
  [24.7],
  [114.514]
)
```

After formatting, it will become:
```typst
#table(
  columns: 3,
  [Substance], [Subcritical °C], [Supercritical °C],
  [Hydrochloric Acid], [12.0], [92.1],
  [Sodium Myreth Sulfate], [16.6], [104],
  [Potassium Hydroxide], [24.7], [114.514],
)
```

## v0.11.21 - [2024-05-16]

Bump to typst v0.11.1

## v0.11.20 - [2024-05-15]

Typstyle cli now include a `format-all` subcommand to format all files in a directory in-place.

```sh
typstyle format-all dir
# or omit the dir to format the current directory
typstyle format-all
```

## v0.11.19 - [2024-05-11]

- Typstyle now indent block math equations.

For example, this code:
```typst
$
E = mc^2
$
```

Now it will be formatted as:
```typst
$
  E = mc^2
$
```

## v0.11.18 - [2024-05-09]

- Typstyle now can keep line comments attached to the end of the line when formatting code blocks.

For example, this code:
```typst
#{
  let c = 0 // my comment
}
```

Previously, the comment will be moved to the next line after formatting. Now it's attached to the end of the line.

```typst
#{
  let c = 0 // my comment
}
```

## v0.11.17 - [2024-05-03]

- Fix typstyle cli not stripping trailing spaces.

## v0.11.16 - [2024-05-01]

- Fix comment loss in closure definition

## v0.11.15 - [2024-04-22]

- Fix comment loss in destruction and set rules

Previously for this code, the comment will be removed after formatting. Now it's kept.

```typst
#let (
// abc
a, b, c,
) = (1, 2, 3)


#set text(
  size: 10pt,
  fallback: false,
  // lang: "de",
)
```

## v0.11.14 - [2024-04-19]

- API Change: allow takes a `typst::Source` as input to avoid re-parsing

## v0.11.13 - [2024-04-12]

- (#49) typstyle cli now support multiple input files. If multiple files are provided, they will be processed in order.
  This is especially useful when you want to format multiple files inplace with a single command.

  ```bash
  typstyle -i **/*.typ
  ```

## v0.11.12 - [2024-04-09]

- Improve performance when formatting nested structures.

Previously it takes infinite time to format this code:
```typst
#let f(..arg) = arg

#f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(f(1,2,3))))))))))))))))))))))
```

Now it is done in instant.

## v0.11.11 - [2024-04-05]

- Fix set rules args are always spread into multiple lines. It now behaves like function call args.

For example, this code:
```typst
#set text(  font: body-font,
  lang: "zh",  region: "cn",
)
```

After formatting, it will become:
```typst
#set text(font: body-font, lang: "zh", region: "cn")
```

- Fix flavor detection for function call args. It now works correctly when the first space in the args contains a newline.

## v0.11.10 - [2024-04-02]

- Block math equations are no longer indented.
- We now support flavor detection for block equations.

For example, this code:
```typst
$
  F(x) = integral_0^x f(t) dif t
$

$ F(x) = integral_0^x f(t) dif t
$

```

After formatting, it will become:
```typst
$
F(x) = integral_0^x f(t) dif t
$

$ F(x) = integral_0^x f(t) dif t $
```

## v0.11.9 - [2024-04-01]

- Trailing spaces are now trimmed.
- Spread args/array/dict into multiple lines if the first space in it contains a newline. This enables flexible control over the formatting of spread args.
  This is called flavor detection.

For example, this code:
```typst
#let my-f(arg1, arg2,
  args: none) = {
  arg1 + arg2
}

#let my-f(arg1,
 arg2, args: none) = {
  arg1 + arg2
}

```

After formatting, it will become:
```typst
#let my-f(arg1, arg2, args: none) = {
  arg1 + arg2
}

#let my-f(
  arg1,
  arg2,
  args: none,
) = {
  arg1 + arg2
}
```

## v0.11.8 - [2024-03-31]

- Fix multiline string/single-backtick-raw-block being wrongly formatted
- Fix missing trailing comma single element array destruct
- Fix `#` is missing in some math environments

## v0.11.7 - [2024-03-30]

- Fix import rename being wrongly formatted

## v0.11.6 - [2024-03-29]

- Fix raw block that starts/ends with backtick is wrongly formatted
- Add version string in `--version` output

## v0.11.5 - [2024-03-28]

- Fix long import item being spread across multiple lines
- Fix bad formatting of destruct items
- Enable formatting when line comment presents in code block

## v0.11.4 - [2024-03-27]

- Put `clap` and `wasm-bindgen` under feature flags to reduce binary size when use as a library

## v0.11.3 - [2024-03-26]

- Nothing new. Just testing ci auto-release

## v0.11.2 - [2024-03-24]

- Fix math attach and function call mis-formatting

## v0.11.1 - [2024-03-21]

- Read from stdin when no arguments are provided

## v0.11.0 - [2024-03-18]

- Initial release
