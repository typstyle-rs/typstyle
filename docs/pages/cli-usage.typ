#import "./book.typ": *

#show: book-page.with(title: "CLI Usage")

#raw(read("../assets/generated/cli-help.txt"), block: true)

= Basic Usage

```bash
# Format multiple files
typstyle -i chapter1.typ chapter2.typ appendix.typ

# Format entire project
typstyle -i .

# Format with specific configuration
typstyle -l 100 -t 4 --wrap-text -i src/
```

= Arguments

== Input Files

```bash
# Format from stdin (default)
cat file.typ | typstyle

# Format specific files
typstyle file1.typ file2.typ

# Format directories (recursively)
typstyle src/ docs/
```

= Main Options

== Format Control

=== In-Place Formatting

```bash
# Modify files directly
typstyle -i file.typ
typstyle --inplace file.typ
```

=== Check Mode

```bash
# Exit with non-zero if formatting needed
typstyle --check src/
```

```bash
# Like --check, but shows unified diff of what formatting changes would be made
typstyle --diff src/
```

=== Line Endings

The default `preserve` policy retains consistent LF or CRLF conventions.
Use `lf` or `crlf` to force generated breaks:

```bash
# Force LF, including when checking in CI
typstyle --line-ending=lf file.typ

# Force CRLF
typstyle --line-ending=crlf --inplace file.typ

# Preserve the existing convention (the default)
typstyle --line-ending=preserve --inplace file.typ
```

`preserve` generates CRLF only when the input contains at least one CRLF and no
bare LF or bare CR. Mixed endings, bare CR, and inputs without newlines fall back
to LF. Detection uses the whole original document,
including for range formatting. All policies select generated layout breaks; newlines in strings,
verbatim raw spans, and format-disabled source remain verbatim. Raw-block boundary
breaks follow the layout policy.
Diff control lines retain their normal LF output.
The default `--check` accepts formatted CRLF files; use `--line-ending=lf` to enforce LF.

== Format Configuration

=== Line Width

```bash
# Set maximum line width (default: 80)
typstyle -l 100 file.typ
typstyle --line-width 100 file.typ
```

=== Indentation

```bash
# Set indentation width (default: 2)
typstyle -t 4 file.typ
typstyle --indent-width 4 file.typ
```

=== Text Wrapping

```bash
# Wrap text in markup to fit line width (`--wrap-text` is shorthand)
typstyle --wrap-text file.typ
typstyle --wrap-text=fill file.typ

# Place each sentence on its own line
typstyle --wrap-text=sentence file.typ

# Place each sentence on its own line, and wrap long sentences to the line width
typstyle --wrap-text=fill-sentence file.typ
```

= Debug Options

== AST Output

```bash
# Print the Abstract Syntax Tree
typstyle -a file.typ
typstyle --ast file.typ
```

== Pretty Document Output

```bash
# Print the internal pretty document representation
typstyle -p file.typ
typstyle --pretty-doc file.typ
```

== Timing Information

```bash
# Show elapsed time taken by the formatter
typstyle --timing file.typ
```

= Logging Options

See CLI help.
