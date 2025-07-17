#let _typstyle = plugin("../assets/typstyle.wasm")

#let default-config = (
  tab_spaces: 2,
  max_width: 80,
  collapse_markup_spaces: false,
  reorder_import_items: true,
  wrap_text: false,
)

#let parse(text) = {
  str(_typstyle.parse(bytes(text)))
}

/// Format text with optional config overrides, panic on failure.
#let format(text, config: (:)) = {
  str(_typstyle.format(bytes(text), bytes(json.encode(default-config + config))))
}

/// Returns none on failure.
#let try-format(text, config: (:)) = {
  let res = _typstyle.try_format(bytes(text), bytes(json.encode(default-config + config)))
  if res.len() == 0 {
    none
  } else {
    str(res)
  }
}

/// Format text, include error as comment on failure.
#let format-with-error(text, config: (:)) = {
  str(_typstyle.format_with_error(bytes(text), bytes(json.encode(default-config + config))))
}

#let format-ir(text, config: (:)) = {
  str(_typstyle.format_ir(bytes(text), bytes(json.encode(default-config + config))))
}
