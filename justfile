plugin_install_path := "contrib/typstyle-embedded/assets/typstyle.wasm"

default:
    @just --list

build-plugin:
    cargo build -p typstyle-typlugin --release --target wasm32-unknown-unknown
    cp target/wasm32-unknown-unknown/release/typstyle_typlugin.wasm {{plugin_install_path}}

generate-cli-help:
    cargo run -p typstyle -- --help \
      | sed '1,/^$/d; s/typstyle\.exe/typstyle/g' \
      > "docs/assets/generated/cli-help.txt"

dev-docs: pre-docs
    shiroa serve docs/pages --root . --mode static-html

build-docs: pre-docs
    shiroa build docs/pages --root . --mode static-html

build-docs-gh: pre-docs
    shiroa build docs/pages --root . --mode static-html --path-to-root /typstyle/

[private]
pre-docs: build-plugin generate-cli-help
