#import "../../deps.typ": shiroa, typstyle
#import shiroa: is-html-target

#let parse-config-from-comment(text) = {
  let lines = text.split("\n")
  if lines.len() == 0 { return (none, text) }

  let first-line = lines.at(0).trim()
  if not first-line.starts-with("/// typstyle:") {
    return (none, text)
  }

  // Extract config part after "/// typstyle:"
  let config-part = first-line.slice(13).trim() // Remove "/// typstyle:"
  let config-changes = (:)

  if config-part.len() > 0 {
    // Split by comma and parse each config item
    let items = config-part.split(",").map(s => s.trim())

    for item in items {
      if item.len() == 0 { continue }

      // Handle simple boolean flags like "wrap_text"
      if not item.contains("=") {
        let key = item
        config-changes.insert(key, true)
      } else {
        // Handle key=value pairs like "max_width=80"
        let parts = item.split("=").map(s => s.trim())
        if parts.len() == 2 {
          let key = parts.at(0)
          let value-str = parts.at(1)

          // Try to parse the value
          let value = if value-str == "true" {
            true
          } else if value-str == "false" {
            false
          } else if value-str.match(regex("^\d+$")) != none {
            int(value-str)
          } else if value-str.match(regex("^\d*\.\d+$")) != none {
            float(value-str)
          } else {
            value-str // Keep as string
          }

          config-changes.insert(key, value)
        }
      }
    }
  }

  // Remove the config comment line from text
  let clean-text = if lines.len() > 1 {
    lines.slice(1).join("\n")
  } else {
    ""
  }

  return (config-changes, clean-text)
}

#let format-config-changes(config-changes) = {
  if config-changes == none or config-changes.len() == 0 {
    return none
  }

  let items = config-changes.pairs().map(((key, value)) => key + ": " + repr(value))

  items.join(", ")
}

#let example(it, config: typstyle.default-config + (max_width: 50)) = {
  // Parse config from special comment
  let (config-changes, clean-text) = parse-config-from-comment(it.text)

  // Apply config changes to the base config
  let final-config = config
  if config-changes != none {
    final-config += config-changes
  }

  // Use clean text (without config comment) for display
  let display-text = clean-text

  let left = raw(display-text, lang: "typ", block: true)
  let right = raw(typstyle.format(display-text, config: final-config), lang: "typ", block: true)

  // Format config changes for display
  let config-info = format-config-changes(config-changes)

  if is-html-target() {
    html.div(class: "example")[
      #html.div(class: "example__header")[
        Example
        #if config-info != none [
          #html.span(class: "example__config")[#config-info]
        ]
      ]
      #html.div(class: "example__content")[
        #html.div(class: "example__panel")[
          #html.div(class: "example__label")[Before]
          #left
        ]
        #html.div(class: "example__panel")[
          #html.div(class: "example__label")[After]
          #right
        ]
      ]
    ]
  } else {
    block(
      fill: rgb("#f8f9fa"),
      stroke: 0.5pt + rgb("#dee2e6"),
      radius: 4pt,
      inset: 0.8em,
      width: 100%,
    )[
      #text(weight: "bold", size: 1.1em)[
        Example
        #if config-info != none [
          #text(size: 0.9em, fill: rgb("#6c757d"))[ (#config-info)]
        ]
      ]
      #grid(
        columns: 2,
        column-gutter: 1em,
        [
          #text(weight: "bold", size: 0.9em)[Before]
          #left
        ],
        [
          #text(weight: "bold", size: 0.9em)[After]
          #right
        ],
      )
    ]
  }
}

#let render-examples(body, lang: "typst") = {
  show raw.where(lang: lang): example
  body
}
