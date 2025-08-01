// This is important for shiroa to produce a responsive layout
// and multiple targets.
#import "../deps.typ": hypraw, shiroa
#import shiroa: plain-text, templates.theme-box
#import "metadata.typ": *
#import "constants.typ": *
#import "rules.typ": code-block-rules, equation-rules, heading-sizes, main-size, markup-rules
#import "components/mod.typ": *


#let web-theme = "starlight"
#let is-starlight-theme = web-theme == "starlight"

#let theme-box = theme-box.with(themes: themes)


/// The project function defines how your document looks.
/// It takes your content and some metadata and formats it.
/// Go ahead and customize it to your liking!
#let project(title: "Typst Book", authors: (), kind: "page", description: none, body) = {
  // set basic document metadata
  set document(author: authors, title: title) if not is-pdf-target

  // todo dirty hack to check is main
  let is-main = title == "Typstyle Documentation"

  // set web/pdf page properties
  set page(numbering: none, number-align: center, width: page-width) if not (sys-is-html-target or is-html-target)
  set page(numbering: "1") if (not sys-is-html-target and is-pdf-target) and not is-main and kind == "page"

  // remove margins for web target
  set page(
    margin: (
      // reserved beautiful top margin
      top: 20pt,
      // reserved for our heading style.
      // If you apply a different heading style, you may remove it.
      left: 20pt,
      // Typst is setting the page's bottom to the baseline of the last line of text. So bad :(.
      bottom: 0.5em,
      // remove rest margins.
      rest: 0pt,
    ),
    height: auto,
  ) if is-web-target and not is-html-target

  show: if is-html-target {
    import "@preview/shiroa-starlight:0.2.3": starlight

    let description = if description != none {
      description
    } else {
      let desc = plain-text(body, limit: 512).trim()
      if desc.len() > 512 {
        desc = desc.slice(0, 512) + "..."
      }
      desc
    }

    starlight.with(
      include "/docs/book.typ",
      title: title,
      site-title: [Typstyle Docs],
      description: description,
      github-link: "https://github.com/typstyle-rs/typstyle",
    )
  } else {
    it => it
  }

  // Set main text
  set text(font: main-font, size: main-size, fill: main-color, lang: "en")

  // markup setting
  show: markup-rules.with(web-theme: web-theme, dash-color: dash-color)
  set heading(numbering: "1.") if is-pdf-target and not is-main

  // math setting
  show: equation-rules.with(theme-box: theme-box)

  // code block setting
  show: code-block-rules.with(themes: themes, code-font: code-font)

  context if shiroa-sys-target() == "html" {
    show raw: it => html.elem("style", it.text)
    ```css
    .pseudo-image svg {
      width: 100%
    }
    ```
  }

  show <typst-raw-func>: it => {
    it.lines.at(0).body.children.slice(0, -2).join()
  }

  if kind == "page" and is-pdf-target and not is-main {
    text(size: 32pt, heading(level: 1, numbering: none, title))
  }

  // Main body.
  set par(justify: true)

  body

  // Put your custom CSS here.
  context if shiroa-sys-target() == "html" {
    html.elem("style", read("styles/base.css"))
    html.elem("style", read("styles/callout.css"))
    html.elem("style", read("styles/example.css"))
    html.elem("style", read("styles/hypraw.css"))
    html.elem("script", read("../packages/hypraw/examples/copy-to-clipboard.js"))
  }
}
