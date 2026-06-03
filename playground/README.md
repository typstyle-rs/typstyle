# Typstyle Playground

An interactive web playground for the [Typstyle](https://github.com/typstyle-rs/typstyle) formatter for Typst documents.

Try it at https://typstyle-rs.github.io/typstyle/playground

## Tech Stack

- **React 19** + **TypeScript** + **Vite**
- **TailwindCSS 4.x** + **DaisyUI** for styling
- **Monaco Editor** with custom Typst language support
- **Typstyle WASM** for client-side formatting
- **pnpm** for package management
- **Biome** for linting and formatting

## Prerequisites

- **Node.js** 18+
- **pnpm** for package management
- **Modern browser** with WebAssembly support

## Quick Start

> **Note:** You must build the WASM module first before running the dev server or building the frontend.

```bash
# Build WASM (run once, or after updating typstyle-wasm)
pnpm dev:wasm         # Build Typstyle WASM module in development mode (faster)
pnpm build:wasm       # Build Typstyle WASM module for production (optimized)

# Development
pnpm dev              # Start Vite development server with hot reload
pnpm preview          # Preview production build locally

# Building
pnpm build            # TypeScript compilation + Vite production build

# Code Quality
pnpm lint             # Run Biome linter to check for issues
pnpm format           # Auto-format code with Biome
pnpm check            # Run Biome linter and formatter together
```

## Integration Details

### WASM Integration

Typstyle is compiled to WebAssembly using `wasm-pack` and loaded directly in the browser for client-side formatting without server dependencies.

### TextMate Grammar

Typst language support is configured with grammar and language definitions adapted from [Tinymist](https://github.com/Myriad-Dreamin/tinymist), providing syntax highlighting and editor features for Typst documents.

Monaco Editor doesn't natively support `.tmLanguage.json` files, so we use `vscode-textmate` (including `vscode-oniguruma`) to convert TextMate grammars into Monaco-acceptable format, enabling rich syntax highlighting and token recognition.

## Credits

- **Typst Language Support**: TextMate grammar and language configuration from [Tinymist](https://github.com/Myriad-Dreamin/tinymist)
- **Design Inspiration**: [Ruff Playground](https://play.ruff.rs/) and [Biome Playground](https://biomejs.dev/playground/)
- **UI Components**: [DaisyUI](https://daisyui.com/) for beautiful Tailwind CSS components
- **Theme Inspiration**: Color palette inspired by Komeiji Koishi

## Contributing

Contributions welcome! Submit issues and PRs to the main [Typstyle repository](https://github.com/typstyle-rs/typstyle).

## License

Apache-2.0 License - see [LICENSE](../LICENSE) for details.
