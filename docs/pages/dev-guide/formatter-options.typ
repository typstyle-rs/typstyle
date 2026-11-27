#import "../book.typ": *
#import callout: *

#show: book-page.with(title: "Adding Formatter Options")

Formatter options have one declaration source: `Options` in
`crates/typstyle-core/src/config.rs`. The `#[options]` attribute generates the partial
configuration type, defaults, merging, validation, CLI arguments, and binding metadata.

= Change Checklist

== Declare The Option

Add a field to `Options` with its default and UI metadata:

```rust
/// Number of spaces per indentation level.
#[option(default = 2)]
#[cli(short = 't', long = "indent-width", visible_alias = "tab-width")]
#[ui(group = "Layout")]
pub tab_spaces: usize,
```

The macro infers the configuration key, label, CLI name, widget, and documentation:

| Property | Default |
| --- | --- |
| Configuration key | Field name |
| CLI long name | Kebab-case field name |
| Description | Rust doc comment |
| Label | Humanized field name |
| UI widget | Boolean → checkbox, integer → number, enum → select, string → text |
| Group | `General` |

Use `#[cli(...)]` to preserve existing flag names or add aliases. Use `#[ui(...)]` to set a
group, widget, or label. Enum-valued fields implement
`typstyle_core::option::OptionEnum`, which supplies the allowed wire values and makes the type
usable by both `serde` and the generated `clap` parser.

Then implement the formatter behavior that reads the field.

== Regenerate Bindings

```bash
cargo xtask generate-options
```

This deterministically writes:

- `packages/options/generated/types.ts` — resolved and partial TypeScript types.
- `packages/options/generated/defaults.json` — canonical defaults.
- `packages/options/generated/schema.json` — JSON Schema with defaults and descriptions.
- `packages/options/generated/ui-schema.json` — groups, labels, widgets, and order.
- `packages/options/generated/options.md` — the configuration reference.

Commit the generated files. CI fails when they are stale:

```bash
cargo xtask generate-options --check
```

Do not generate these files from `build.rs`.

== Configuration Files

TOML files deserialize into `PartialOptions`; unknown keys are rejected. The CLI merges layers
from lowest to highest priority:

+ built-in defaults,
+ the user configuration (`$TYPSTYLE_CONFIG`, then `$XDG_CONFIG_HOME/typstyle/config.toml`),
+ the project configuration (`--config`, then `typstyle.toml` or `.typstyle.toml`),
+ explicit CLI flags.

Defaults are applied only after merging. Unknown keys are rejected, and no field-level
ranges are currently declared. Explicit values — including `false` and values equal to a
default — always override lower layers.

== CLI

Generated arguments come from `PartialOptions`; do not duplicate field declarations in
`crates/typstyle/src/cli.rs`. Preserve an existing flag by setting it in `#[cli(...)]`.

Handwritten compatibility flags are allowed when a historical spelling cannot be expressed as
a generated option, such as the inverse `--no-reorder-import-items` switch. Map it into
`PartialOptions` in `StyleArgs::to_partial`.

Booleans accept explicit values (`--reorder-import-items=false`) so they can override a
configuration file. Do not add generated bare boolean switches.

Run `just generate-cli-help` to refresh the long help used by the documentation.
`just sync-readme-cli-help` refreshes the short usage block in `README.md`, and
`just check-cli-help` verifies that the block is current.

== Playground And UI

The playground edits explicit overrides, not resolved values. `OptionsEditor` reads
`ui-schema.json` and renders ordinary controls by type and widget. Resetting a field removes
its override; resetting all clears the override object.

Use the `renderers` prop for an exceptional control:

```tsx
<OptionsEditor
  value={overrides}
  onChange={setOverrides}
  renderers={{ wrap_mode: CustomWrapModeControl }}
/>
```

== Tests

Add focused semantic tests for formatter behavior and configuration layering:

+ core: defaults, merging, explicit false overrides, serde round trips;
+ CLI: project/user/CLI precedence, unknown keys, preserved and new flags;
+ playground: WASM binding shape with generated keys, URL override validation.

Before considering the change complete:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
cargo xtask generate-options --check
cd playground && pnpm test:run && pnpm build
```
