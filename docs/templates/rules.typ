#import "../deps.typ": shiroa, hypraw
#import shiroa.templates: *
#import "metadata.typ": *
#import "constants.typ": *

/// Creates an embedded block typst frame.
#let div-frame(content, attrs: (:), tag: "div") = html.elem(tag, html.frame(content), attrs: attrs)
#let span-frame = div-frame.with(tag: "span")
#let p-frame = div-frame.with(tag: "p")


#let markup-rules(
  body,
  dash-color: none,
  web-theme: "starlight",
  main-size: main-size,
  heading-sizes: heading-sizes,
  list-indent: list-indent,
  starlight: "@preview/shiroa-starlight:0.2.3",
) = {
  assert(dash-color != none, message: "dash-color must be set")

  let is-starlight-theme = web-theme == "starlight"
  let in-heading = state("shiroa:in-heading", false)

  let mdbook-heading-rule(it) = {
    let it = {
      set text(size: heading-sizes.at(it.level))
      if is-web-target {
        heading-hash(it, hash-color: dash-color)
      }

      in-heading.update(true)
      it
      in-heading.update(false)
    }

    block(
      spacing: 0.7em * 1.5 * 1.2,
      below: 0.7em * 1.2,
      it,
    )
  }

  let starlight-heading-rule(it) = context if shiroa-sys-target() == "html" {
    import starlight: builtin-icon

    in-heading.update(true)
    html.elem("div", attrs: (class: "sl-heading-wrapper level-h" + str(it.level + 1)))[
      #it
      #html.elem(
        "h" + str(it.level + 1),
        attrs: (class: "sl-heading-anchor not-content", role: "presentation"),
        static-heading-link(it, body: builtin-icon("anchor"), canonical: true),
      )
    ]
    in-heading.update(false)
  } else {
    mdbook-heading-rule(it)
  }


  // Set main spacing
  set enum(
    indent: list-indent * 0.618,
    body-indent: list-indent,
  )
  set list(
    indent: list-indent * 0.618,
    body-indent: list-indent,
  )
  set par(leading: 0.7em)
  set block(spacing: 0.7em * 1.5)

  // Set text, spacing for headings
  // Render a dash to hint headings instead of bolding it as well if it's for web.
  show heading: set text(weight: "regular") if is-web-target
  // todo: add me back in mdbook theme!!!
  show heading: if is-starlight-theme {
    starlight-heading-rule
  } else {
    mdbook-heading-rule
  }

  // link setting
  show link: set text(fill: dash-color)

  body
}

#let equation-rules(
  body,
  web-theme: "starlight",
  theme-box: none,
) = {
  // import "supports-html.typ": add-styles
  let is-starlight-theme = web-theme == "starlight"
  let in-heading = state("shiroa:in-heading", false)

  /// Creates an embedded block typst frame.
  let div-frame(content, attrs: (:), tag: "div") = html.elem(tag, html.frame(content), attrs: attrs)
  let span-frame = div-frame.with(tag: "span")
  let p-frame = div-frame.with(tag: "p")


  let get-main-color(theme) = {
    if is-starlight-theme and theme.is-dark and in-heading.get() {
      white
    } else {
      theme.main-color
    }
  }

  show math.equation: set text(weight: 400)
  show math.equation.where(block: true): it => context if shiroa-sys-target() == "html" {
    theme-box(tag: "div", theme => {
      set text(fill: get-main-color(theme))
      p-frame(attrs: ("class": "block-equation", "role": "math"), it)
    })
  } else {
    it
  }
  show math.equation.where(block: false): it => context if shiroa-sys-target() == "html" {
    theme-box(tag: "span", theme => {
      set text(fill: get-main-color(theme))
      span-frame(attrs: (class: "inline-equation", "role": "math"), it)
    })
  } else {
    it
  }

  // add-styles(
  //   ```css
  //   .inline-equation {
  //     display: inline-block;
  //     width: fit-content;
  //   }
  //   .block-equation {
  //     display: grid;
  //     place-items: center;
  //     overflow-x: auto;
  //   }
  //   ```,
  // )
  body
}

#let with-raw-theme = (theme, it) => {
  if theme.len() > 0 {
    raw(
      align: it.align,
      tab-size: 2,
      block: it.block,
      lang: it.lang,
      syntaxes: it.syntaxes,
      theme: theme,
      it.text,
    )
  } else {
    raw(
      align: it.align,
      tab-size: 2,
      block: it.block,
      lang: it.lang,
      syntaxes: it.syntaxes,
      theme: auto,
      it.text,
    )
  }
}

#let code-block-rules(
  body,
  web-theme: "starlight",
  code-font: none,
  themes: none,
) = {
  let (
    default-theme: (
      style: theme-style,
      is-dark: is-dark-theme,
      is-light: is-light-theme,
      main-color: main-color,
      dash-color: dash-color,
      code-extra-colors: code-extra-colors,
    ),
  ) = themes
  let (
    default-theme: default-theme,
  ) = themes
  let theme-box = theme-box.with(themes: themes)

  /// HTML code block supported by hypraw.
  show: hypraw.hypraw
  set raw(tab-size: 114)

  let in-mk-raw = state("shiroa:in-mk-raw", false)
  let mk-raw(it, tag: "div", inline: false) = {
    theme-box(tag: tag, theme => {
      set par(justify: false)
      with-raw-theme(theme.style.code-theme, it)
    })
  }

  show raw: set text(font: code-font) if code-font != none
  show raw.where(block: false, tab-size: 114): it => context if shiroa-sys-target() == "paged" {
    it
  } else {
    mk-raw(it, tag: "span", inline: true)
  }
  show raw.where(block: true, tab-size: 114): it => context if shiroa-sys-target() == "paged" {
    rect(width: 100%, inset: (x: 4pt, y: 5pt), radius: 4pt, fill: code-extra-colors.bg, {
      set text(fill: code-extra-colors.fg) if code-extra-colors.fg != none
      set par(justify: false)
      with-raw-theme(theme-style.code-theme, it)
    })
  } else {
    mk-raw(it)
  }
  body
}
