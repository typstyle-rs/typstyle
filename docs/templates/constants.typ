#import "../deps.typ": shiroa
#import shiroa: templates.theme-box-styles-from
#import "metadata.typ": is-web-target

// Theme (Colors)
#let themes = theme-box-styles-from(toml("theme-style.toml"), xml: it => xml(it))
#let (
  default-theme: (
    style: theme-style,
    is-dark: is-dark-theme,
    is-light: is-light-theme,
    main-color: main-color,
    dash-color: dash-color,
    code-extra-colors: code-extra-colors,
  ),
) = themes;
#let (
  default-theme: default-theme,
) = themes;

// Sizes
#let main-size = if is-web-target {
  16pt
} else {
  10.5pt
}
#let heading-sizes = if is-web-target {
  (2, 1.5, 1.17, 1, 0.83).map(it => it * main-size)
} else {
  (26pt, 22pt, 14pt, 12pt, main-size)
}
#let list-indent = 0.5em

// Fonts
#let main-font = (
  // "Charter",
  "Source Han Serif SC",
  // "Source Han Serif TC",
  "Libertinus Serif",
)
#let code-font = (
  // "BlexMono Nerd Font Mono",
  "DejaVu Sans Mono",
)
