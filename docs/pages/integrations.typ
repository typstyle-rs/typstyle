#import "./book.typ": *

#show: book-page.with(title: "Integrations")

Typstyle integrates with editors, continuous integration, and version-control workflows.

= Editor Integration

Typstyle has been integrated into #link("https://github.com/Myriad-Dreamin/tinymist")[tinymist]. You can use it in your editor by installing the tinymist plugin and set `tinymist.formatterMode` to `typstyle`.

== VS Code (via Tinymist)

+ Install the #link("https://marketplace.visualstudio.com/items?itemName=myriad-dreamin.tinymist")[Tinymist extension]
+ Set `tinymist.formatterMode` to `"typstyle"` in your settings
+ Enable format on save or use `Ctrl+Shift+P` → "Format Document"

= GitHub Actions

The #link("https://github.com/typstyle-rs/typstyle-action")[typstyle-action] maintained by #link("https://github.com/grayespinoza")[grayespinoza] can install and run Typstyle in a GitHub Action.

```yaml
- name: Run typstyle
  uses: typstyle-rs/typstyle-action@main
```

= Pre-commit Hook

Add this to your `.pre-commit-config.yaml`:

```yaml
  - repo: https://github.com/typstyle-rs/pre-commit-typstyle
    rev: ''  # The revision or tag you want to use
    hooks:
      - id: typstyle
```

= Working with Jujutsu

Typstyle works with #link("https://github.com/jj-vcs/jj")[Jujutsu] through its `jj fix` command.
`jj fix` does not run formatters on paths in the working copy: it writes the content of each selected file to stdin and reads the formatted content from stdout.
Use the plain `typstyle` command as the fix tool:

```toml
[fix.tools.typstyle]
command = ["typstyle"]
patterns = ["glob:'**/*.typ'"]
```

Style options can be passed the same way, for example `command = ["typstyle", "--line-width=100"]`.

Do not add `-i` to the command: in-place mode requires file arguments and is rejected when typstyle reads from stdin.
Do not pass `$path` either, since a path argument makes typstyle read the working-copy file and ignore stdin.
A file that cannot be parsed is echoed unchanged with a successful exit code, so `jj fix` leaves it intact.

Because `jj fix` selects files by #link("https://docs.jj-vcs.dev/latest/filesets/")[fileset], its selection is independent of typstyle's directory walking: tracked Typst files under dot-directories are formatted as well.
When you run typstyle directly in a Jujutsu workspace, recursive directory walks skip entries whose names start with a dot, so `.jj` and the `.git` directory of a colocated repository are never touched.

#callout.note[
  `line-range-args` is not supported because the typstyle CLI has no line-range option. `jj fix` therefore formats whole files.
]
