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
