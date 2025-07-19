# typstyle-wasm

[![npm wasm](https://img.shields.io/npm/v/@typstyle/typstyle-wasm-bundler)](https://www.npmjs.com/package/@typstyle/typstyle-wasm-bundler)

WebAssembly bindings for [Typstyle](https://github.com/typstyle-rs/typstyle), a beautiful and reliable [Typst](https://typst.app/) code formatter.

> ⚠️ **API Stability Warning**: The API is unstable and may change in any future version.

## Installation

```bash
# For bundlers (Webpack, Vite, etc.)
npm install @typstyle/typstyle-wasm-bundler
```

## Usage

### Bundler Target

```javascript
import { format } from "@typstyle/typstyle-wasm-bundler";

const formatted = format("#let x=1", {}); // Format with default config
```

## API

- `format(text: string, config: Config): string` - Format Typst code
- `format_ir(text: string, config: Config): string` - Return formatting IR
- `parse(text: string): string` - Parse code and return AST debug string

## Configuration

See the auto-generated docstring of `Config` type.

## License

Apache License 2.0. See [LICENSE](./../../LICENSE).
