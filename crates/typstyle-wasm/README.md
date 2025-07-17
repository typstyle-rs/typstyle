# typstyle-wasm

WebAssembly bindings for [Typstyle](https://github.com/typstyle-rs/typstyle), a beautiful and reliable [Typst](https://typst.app/) code formatter.

> ⚠️ **API Stability Warning**: The API is unstable and may change in any future version.

## API

- `format(text: string, config: Config): string` - Format Typst code
- `format_ir(text: string, config: Config): string` - Return formatting IR
- `parse(text: string): string` - Parse code and return AST debug string

## Configuration

See the auto-generated docstring of `Config` type.

## License

Apache License 2.0. See [LICENSE](./LICENSE).
