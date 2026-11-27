#import "../book.typ": *
#import callout: *

#show: book-page.with(title: "Adding Formatter Options")

Formatter options have one declaration source: `options.toml` at the repository root.
`cargo xtask generate-options` writes ordinary Rust, TypeScript, JSON, and Markdown from
that file. Generated files are committed; no procedural macro or extra published crate is
involved.

= Change Checklist

== Declare The Option

Add an entry to `options.toml` with its type, default, documentation, and UI metadata:

```toml
[[option]]
name = "tab_spaces"
type = "usize"
default = 2
description = "Number of spaces to use for each indentation level."
group = "Layout"
label = "Indent width"
cli = { short = "t", long = "indent-width", visible_alias = "tab-width", value_name = "INDENT_WIDTH" }
```

The generator infers everything that does not require explicit specification:

| Property | Default |
| --- | --- |
| Rust field | `name` |
| Configuration key | `name` |
| CLI long name | Kebab-case `name` |
| CLI help | `description` plus the default |
| Label | Humanized `name` |
| UI widget | Boolean → checkbox, integer → number, enum → select, string → text |
| Group | `General` |

Use `cli` to preserve an existing flag name, add aliases, or declare an inverse switch.
Enum-valued options reference an `[[enum]]` table, which supplies the allowed wire values,
variant documentation, and the serde representation:

```toml
[[enum]]
name = "WrapMode"
description = "Text wrapping mode for markup."
default = "none"
variants = [
  { name = "None", value = "none", description = "Do not wrap text." },
  { name = "Fill", value = "fill", description = "Wrap text to fit within the line width." },
]
```

Then implement the formatter behavior that reads the field. Semantic validation that spans
fields belongs in `Options::validate` in `crates/typstyle-core/src/config.rs`; keep
`options.toml` declarative.

== Regenerate Bindings

```bash
cargo xtask generate-options
```

This deterministically writes:

- `crates/typstyle-core/src/config/generated.rs` — `Options`, `PartialOptions`, option
  enums, defaults, merging, and `OptionSpec` metadata.
- `crates/typstyle/src/generated_args.rs` — the `clap` argument struct and conversion into
  `PartialOptions`.
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

== Generated Rust

The generated core module contains ordinary source code, so IDE navigation and documentation
work normally. The core crate does not depend on `clap`; CLI-specific arguments are generated
in the CLI crate. Both generated files are checked in and regenerated explicitly, never
during a downstream build.

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

Generated arguments come from `OptionArgs`, which converts into `PartialOptions`. Do not
duplicate option declarations in `crates/typstyle/src/cli.rs`. Preserve a historical spelling
with `cli.long`, `cli.short`, or the alias fields; express an inverse switch with
`cli.inverse` rather than a handwritten option.

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
