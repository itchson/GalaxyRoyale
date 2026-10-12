import { defineConfig } from "vite";
import { resolve } from "node:path";
import { readWasmBuilds } from "../../scripts/wasm-artifacts.mjs";
import { singleGameRuntime } from "../../scripts/game-runtime-lease.mjs";
export default defineConfig({
  plugins: [singleGameRuntime()],
  root: resolve(import.meta.dirname),
  define: {
    // An external preview needs an explicitly configured signaling service.
    // Local desktop testing chooses loopback in the shell; no endpoint or
    // credentials are silently embedded in the public game bundle.
    __COOP_SERVICE__: JSON.stringify(process.env.GALAXY_COOP_SERVICE ?? null),
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
