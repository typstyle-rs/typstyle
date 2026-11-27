/// <reference types="vitest/config" />

import * as path from "node:path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import { viteStaticCopy } from "vite-plugin-static-copy";

// https://vite.dev/config/
export default defineConfig({
  base: "/typstyle/playground/",

  // Vitest's jsdom suite uses the client environment but executes in Node.
  // Let Vite read WASM from disk instead of fetching a browser asset URL.
  environments: process.env.VITEST
    ? { client: { consumer: "server" } }
    : undefined,

  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
      "@typstyle/options": path.resolve(
        import.meta.dirname,
        "../packages/options/src/index.ts",
      ),
    },
  },

  plugins: [
    react(),
    tailwindcss(),
    viteStaticCopy({
      targets: [
        {
          src: "../tests/fixtures/ai/**/*.typ",
          dest: "samples",
          rename: { stripBase: true },
        },
      ],
    }),
  ],

  optimizeDeps: {
    exclude: ["typstyle-wasm"],
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id): string | undefined => {
          // Large packages get their own chunks
          if (id.includes("monaco-editor")) {
            return "monaco-editor";
          }
          if (id.includes("monaco-themes")) {
            return "monaco-themes";
          }
          if (id.includes("react-dom")) {
            return "react-dom";
          }
          if (id.includes("react")) {
            return "react";
          }
          if (id.includes("typstyle-wasm")) {
            return "typstyle-wasm";
          }
          if (id.includes("node_modules")) {
            return "vendor";
          }
        },
      },
    },
  },

  test: {
    environment: "jsdom",
  },
});
