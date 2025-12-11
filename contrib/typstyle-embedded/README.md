# Typstyle Embedded

A Typst package that embeds the Typstyle formatter directly into your documents using WASM plugin.

## Installation

It is not published. So simply copy it to your project or install it into local packages.

## Usage

### Basic Formatting

```typst
#let code = "#let x=1+2"
#let formatted = typstyle.format(code)
// Result: "#let x = 1 + 2"
```

### With Custom Configuration

```typst
#let my-config = (
  tab_spaces: 4,
  max_width: 120,
)

#let formatted = typstyle.format(code, config: my-config)
```

## API Reference

### Formatting Functions

- `format(text, config: (:))` - Format text with optional config overrides, panic on failure.
- `try-format(text, config: (:))` - Try to format, returns `none` on failure.
- `format-with-error(text, config: (:))` - Format text, include error as comment on failure.

### Utility Functions

- `parse(text)` - Parse text and return AST
- `format-ir(text, config: (:))` - Get formatting intermediate representation

## Default Configuration

This follows the defaults of the Rust struct. The fields and values are subject to change in future versions.

```typc
(
  tab_spaces: 2,
  max_width: 80,
  collapse_markup_spaces: false,
  reorder_import_items: true,
  wrap_text: false,
)
```

## Error Handling

The package provides different functions for various error handling scenarios:

- **`format()`** - Standard formatting that can throw errors
- **`try-format()`** - Safe formatting that returns `none` on failure
- **`format-with-error()`** - Includes error information as comments when formatting fails

### Usage Examples

```typst
// Standard formatting (may error)
#let result = format(code)

// Safe formatting with fallback
#let result = try-format(code)
#if result == none [
  Failed to format code
] else [
  #result
]

// Format with error preservation
#let result = format-with-error(code)
// If formatting fails, original code is returned with error comment
```

## License

Same as the main Typstyle project.
