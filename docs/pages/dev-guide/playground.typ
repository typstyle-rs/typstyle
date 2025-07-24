#import "../book.typ": *
#import callout: *

#show: book-page.with(title: "Playground Development")

The #link(package.homepage + "playground")[typstyle playground] is an interactive web application for trying typstyle formatting in the browser.

= Tech Stack

- *React 19* + *TypeScript* + *Vite*
- *TailwindCSS 4.x* + *DaisyUI* for styling
- *Monaco Editor* with custom Typst language support
- *Typstyle WASM* for client-side formatting
- *pnpm* for package management

= Quick Start

Prerequisites: Node.js 18+ and pnpm

```bash
cd playground

# Development
pnpm install        # Install dependencies
pnpm dev            # Start development server with hot reload
pnpm preview        # Preview production build locally

# Building
pnpm build          # Production build
pnpm build:wasm     # Build Typstyle WASM module

# Code Quality
pnpm check          # Run linter and formatter
```

= Key Components

- *WASM Integration*: Typstyle compiled to WebAssembly for client-side formatting
- *Monaco Editor*: Custom Typst language support with TextMate grammar from #link("https://github.com/Myriad-Dreamin/tinymist")[Tinymist]

#important[
  Rebuild WASM bindings (```sh pnpm dev:wasm``` or ```sh pnpm build:wasm```) after changes to `typstyle-core` or `typstyle-wasm` crates.
]

= Development Workflow

1. *Code changes*: Modify TypeScript/React code in `src/`
2. *WASM changes*: Rebuild WASM if Rust code changed
3. *Test*: Use ```sh pnpm dev``` for development server
4. *Deploy*: Build with ```sh pnpm build``` and deploy `dist/` as static site
