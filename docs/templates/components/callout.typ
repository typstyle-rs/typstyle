#import "../../deps.typ": shiroa
#import shiroa: is-html-target, is-web-target

/// Creates a callout box with different styles for different targets
///
/// Parameters:
/// - type: "note", "important", "warning", "tip", "caution"
/// - title: Optional custom title (defaults to capitalized type)
/// - body: The content to display in the callout
#let callout(type: "note", title: none, body) = {
  // Define colors and icons for different callout types
  let callout-config = (
    note: (
      color: rgb("#0969da"),
      bg-color: rgb("#ddf4ff"),
      border-color: rgb("#0969da"),
      icon: "ℹ",
      default-title: "Note",
    ),
    important: (
      color: rgb("#8250df"),
      bg-color: rgb("#fbf0ff"),
      border-color: rgb("#8250df"),
      icon: "❗",
      default-title: "Important",
    ),
    warning: (
      color: rgb("#d1242f"),
      bg-color: rgb("#ffebee"),
      border-color: rgb("#d1242f"),
      icon: "⚠",
      default-title: "Warning",
    ),
    tip: (
      color: rgb("#1a7f37"),
      bg-color: rgb("#dcffe4"),
      border-color: rgb("#1a7f37"),
      icon: "💡",
      default-title: "Tip",
    ),
    caution: (
      color: rgb("#bf8700"),
      bg-color: rgb("#fff8c5"),
      border-color: rgb("#bf8700"),
      icon: "⚠",
      default-title: "Caution",
    ),
  )

  let config = callout-config.at(type, default: callout-config.note)
  let display-title = if title != none { title } else { config.default-title }

  if is-html-target() {
    // HTML/Web target: Use CSS classes instead of inline styles
    html.div(class: "callout callout-" + type, {
      html.div(class: "callout-title", {
        html.span(config.icon)
        html.span(display-title)
      })
      html.div(class: "callout-content", body)
    })
  } else {
    // PDF/Print target: Use Typst native styling
    block(
      width: 100%,
      fill: config.bg-color,
      stroke: (left: 4pt + config.border-color),
      radius: 4pt,
      inset: 12pt,
      {
        // Title with icon
        text(
          fill: config.color,
          weight: "bold",
          size: 1.1em,
          [#config.icon #h(0.5em) #display-title],
        )

        // Add spacing between title and content
        v(0.5em)

        // Content
        set text(fill: black)
        body
      },
    )
  }
}

/// Convenience functions for common callout types
#let note(title: none, body) = callout(type: "note", title: title, body)
#let important(title: none, body) = callout(type: "important", title: title, body)
#let warning(title: none, body) = callout(type: "warning", title: title, body)
#let tip(title: none, body) = callout(type: "tip", title: title, body)
#let caution(title: none, body) = callout(type: "caution", title: title, body)
