#import "deps.typ": shiroa
#import shiroa: *

#show: book

#book-meta(
  title: "typstyle",
  description: "Typstyle Documentation",
  repository: "https://github.com/typstyle-rs/typstyle",
  repository-edit: "https://github.com/typstyle-rs/typstyle/edit/master/docs/pages/{path}",
  summary: [
    #prefix-chapter("introduction.typ")[Introduction]
    = User Guide
    - #chapter("installation.typ")[Installation]
    - #chapter("quick-start.typ")[Quick Start]
    - #chapter("changelog.typ")[Changelog]
    = Usage
    - #chapter("cli-usage.typ")[Command Line Interface]
    - #chapter(none)[Editor Integration]
    = Features
    - #chapter("features.typ")[Formatting Features]
      - #chapter("features/markup.typ")[Markup]
      - #chapter("features/code.typ")[Code]
      - #chapter("features/math.typ")[Math Equations]
      - #chapter("features/table.typ")[Tables]
    - #chapter("escape-hatch.typ")[Escape Hatch]
    - #chapter("limitations.typ")[Limitations]
    = Advanced
    - #chapter("architecture.typ")[How It Works]
    - #chapter("dev-guide.typ")[Developer Guide]
      - #chapter("dev-guide/core.typ")[Core]
      - #chapter("dev-guide/documentation.typ")[Documentation]
      - #chapter("dev-guide/playground.typ")[Playground]
      - #chapter("dev-guide/release-process.typ")[Release Process]
  ],
)

#build-meta(dest-dir: "../dist")

// re-export page template
#import "templates/page.typ": project
#let book-page = project

// re-export components
#import "templates/components/mod.typ": render-examples
#import "templates/components/callout.typ"
