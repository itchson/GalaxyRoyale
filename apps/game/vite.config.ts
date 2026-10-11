import { defineConfig } from "vite";
import { resolve } from "node:path";
import { readWasmBuilds } from "../../scripts/wasm-artifacts.mjs";
export default defineConfig({
  root: resolve(import.meta.dirname),
  define: {
    __WASM_BUILDS__: JSON.stringify(
      readWasmBuilds(resolve(import.meta.dirname, "../..")),
    ),
  },
  server: { host: "127.0.0.1", port: 4173, strictPort: true },
  preview: { host: "127.0.0.1", port: 4173, strictPort: true },
  build: {
    outDir: resolve(import.meta.dirname, "../../dist"),
    emptyOutDir: true,
    target: "es2022",
  },
});
