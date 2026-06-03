# typstyle-wasm

[![npm wasm](https://img.shields.io/npm/v/@typstyle/typstyle-wasm-bundler)](https://www.npmjs.com/package/@typstyle/typstyle-wasm-bundler)

WebAssembly bindings for [Typstyle](https://github.com/typstyle-rs/typstyle), a beautiful and reliable [Typst](https://typst.app/) code formatter.

> ⚠️ **API Stability Warning**: The API is unstable and may change in any future version.

## Building

### Prerequisites

- Rust stable toolchain with the `wasm32-unknown-unknown` target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)

```bash
rustup target add wasm32-unknown-unknown
cargo binstall wasm-pack
```

### For the Playground

The playground lives in the `playground/` directory and uses a local wasm build:

```bash
cd playground
pnpm build:wasm       # Production build (optimized)
pnpm dev:wasm         # Development build (with debug info, faster compile)
```

This runs `wasm-pack build` against this crate and outputs to `playground/typstyle-wasm/`.

### For NPM Publishing

```bash
cd crates/typstyle-wasm
wasm-pack build --target bundler --out-dir pkg --scope typstyle
```

This produces the `@typstyle/typstyle-wasm-bundler` package in `crates/typstyle-wasm/pkg/`.

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
