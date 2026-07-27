#import "../book.typ": *
#import callout: *

#show: book-page.with(title: "Adding Formatter Options")

Formatter options cross crate and application boundaries. Changing only core `Config` can leave the CLI, WASM bindings, embedded package, playground, or docs with stale values.

= Change Checklist

== Core Configuration

+ Add the field to `crates/typstyle-core/src/config.rs`, set its `Config::default()` value, and add a builder method when useful to library callers.
+ Update every formatter path that reads it; search for the field and related builder methods.

#important[
  Adding, removing, or changing a public `Config` field changes the Rust API. Decide whether the release may be breaking and migrate every in-workspace consumer together.
]

== CLI

+ Add the argument/value enum in `crates/typstyle/src/cli.rs` and map it to core `Config` in `crates/typstyle/src/fmt.rs`.
+ Preserve existing invocations when extending a boolean flag into an optional value. Require `=` for values so paths are not consumed: `--wrap-text=sentence`, while `--wrap-text file.typ` remains valid.
+ Test positional paths and invalid values in `crates/typstyle/tests/test_style_args.rs`.
+ Run `just generate-cli-help` and update copied help text in `README.md`.

== Tests and Rust Consumers

+ Update direct `Config` literals in `crates/` and `tests/`; update `tests/src/common/directive.rs` if fixtures need to select the option.
+ Add focused semantic tests and representative fixture snapshots. Keep convergence and consistency checks enabled unless disabling them is understood and documented.

Before considering migration complete, search the repository and check the workspace:

```bash
rg 'old_field|OldVariant' --glob '!target/**'
cargo check --workspace
```

== Serialized, Embedded, and Documentation Consumers

`Config` is deserialized by `typstyle-wasm` and `typstyle-typlugin`. For serialized options:

+ Test serde serialization and deserialization in `typstyle-core`.
+ Ensure `crates/typstyle-wasm/build.rs` translates the field type into the generated TypeScript `Config` interface and honors serde names instead of Rust variant names.
+ Update defaults in `contrib/typstyle-embedded/src/lib.typ` and its README.
+ Update `docs/pages/` render directives: examples use embedded-plugin configuration, not CLI argument names. Run `just build-plugin` before building docs.

== Playground

+ Update `FormatOptions`, defaults, and `formatOptionsToConfig` in `playground/src/utils/formatter.ts`, plus the control in `playground/src/components/forms/SettingsPanel.tsx`.
+ Update URL-state expectations if the option name or type changed, and add a WASM binding test for its serialized value.
+ Rebuild generated bindings before TypeScript validation:
  ```bash
  cd playground
  pnpm build:wasm
  pnpm build
  pnpm test:run
  ```

= Final Validation

Run checks appropriate to every touched boundary:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features
cargo nextest run --workspace --no-fail-fast
just build-docs
```

For behavior changes, review snapshots and test narrow widths, inline markup, comments, line-sensitive markup markers, and repeated formatting for convergence.
