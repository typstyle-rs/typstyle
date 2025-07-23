import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import react from "@vitejs/plugin-react";
import toplevelAwait from "vite-plugin-top-level-await";
import tailwindcss from "@tailwindcss/vite";
import * as path from "path";

// https://vite.dev/config/
export default defineConfig({
  base: "/typstyle/playground/",

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
    },
  },

  plugins: [
    react(),
    tailwindcss(),
    wasm(),
    toplevelAwait(), // required by wasm
  ],

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
});
