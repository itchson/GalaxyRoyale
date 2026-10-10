# HUD/Riveter local packaging review

Final audit time: 2026-10-10 19:07 UTC. Refreshed after the 200-pixel safe-width native painter/reservation guard fix and its normal rebuild. This supersedes the earlier 19:01–19:03 package snapshot. Read-only inspection of the completed local `dist`, generated public WASM, build scripts/log and `.openai/hosting.json`. No publishing, credentials, authenticated remote execution or broad test reruns were performed.

## Result

**Local package accepted.** The normal WebGPU and WebGL2 outputs are present, QA checkpoint fixtures are excluded, the exact 18-asset game cache matches the actual package, and the independent studio/production data is excluded. Hosting selects the game `dist` directory.

Current cache is **`galaxy-f37b303ee2258928`**. Recomputing the build script's sorted relative-path-plus-file-bytes SHA256 reproduces that ID. `artifacts/hud-riveter-build-final.log` records the successful normal dual-WASM, TypeScript, game Vite/PWA and separate studio build. The full log SHA256 is `aa22ed0f8a95e3aac883645a875d863a63571730033e6c7877bbe7c79d5e6043`. The old `galaxy-4d3a5fcb06f66cd8` package is not the final acceptance target.

This is local package acceptance. `.openai/hosting.json` contains a project binding and `static.directory="dist"`; it does not encode an authentication policy. Owner-authenticated private remote access, current remote bytes and live remote execution require root's separate evidence. No private remote execution result is inferred from a local hash or hosting binding.

## Normal WASM and QA exclusion

Both binaries successfully parsed as actual `WebAssembly.Module` instances in Node for export inspection; no GPU context or game instance was created. WebGPU has 48 exports and WebGL2 has 44. Both contain the normal production `boss_ui_bounds` export. **Neither exports `checkpoint_contract`**, and that fixture name is absent from both binary bytes, their generated JavaScript/type declarations and the bundled game shell.

`scripts/build-wasm.mjs` uses locked release builds with `--no-default-features` and only `webgpu` or `webgl`. It does not enable `qa-contract`. All eight generated backend files in `dist/wasm/` match the corresponding `apps/game/public/wasm/` files byte-for-byte.

| Backend / file | Bytes | SHA256 (dist and public identical) |
| --- | --- | --- |
| WebGPU `galaxy_bg.wasm` | 2,048,967 | `f2cf269dd8ae35dbddf62f630d51b44d452f96fbac14175ba95bbb077e8575f8` |
| WebGPU `galaxy.js` | 77,645 | `c1f0fe72291fc2713b9165de3dffdc470355386b6f31caa7756db14447711187` |
| WebGPU `galaxy.d.ts` | 7,320 | `c1a6f93bb491d2b5565d37d46e80a3fd9ee74eb9785f8c0b49d13c7d49b331fb` |
| WebGPU `galaxy_bg.wasm.d.ts` | 4,630 | `f2b9f908eb1a4692d43f4fe9c510f24a115c8057d28e037b5898956dbe74b8b9` |
| WebGL2 `galaxy_bg.wasm` | 5,696,084 | `93a7daa913c8cc3ebc4b1c59de5d5d4bff374ddacc83fd2dfbd6af3bb5e2374c` |
| WebGL2 `galaxy.js` | 80,969 | `327185a19e7a9a959f377c77a5d99eb159c20295a55e5ed4845df6612380d123` |
| WebGL2 `galaxy.d.ts` | 6,054 | `dcff0b67f31bf5f46ddc07ab386e5654c6654f5d14e585dcbe003680bc1db032` |
| WebGL2 `galaxy_bg.wasm.d.ts` | 3,364 | `05ccf14d3ec3170f91aa338e7c23ba95e68d1b3f05cd79b5ee0ec1e878df0926` |

## Game-only PWA and studio separation

The service worker's 18 asset URLs exactly equal every actual file under `dist` except `sw.js`; none is missing. They are the game HTML, hashed JS/CSS, manifest, three icon files, three font/license files and eight normal WASM backend files. No studio, production/project JSON, work evidence, QA fixture, hosting metadata or credential file is in that list/package. The bundled game shell has no studio API/project endpoint reference.

The worker excludes `/api/` and `/studio` fetches, caches only listed same-origin game assets, and activates through the existing explicit `ACTIVATE_AT_DOCK` message. Navigation uses network with the cached game HTML as offline fallback. This source/package review verifies cache scope and activation plumbing; current-build offline behavior and physical PWA lifecycle still require execution evidence.

Studio output exists independently at `apps/studio/dist/index.html`. The studio Vite config's root-relative `dist` output does not overwrite the root game `dist`. There is no `dist/studio` folder or published `dist/production/project.json`. The hosting config resolves `static.directory` to the root game output.

| Packaged metadata | SHA256 |
| --- | --- |
| `dist/sw.js` | `25cf638532a79e9a99ae19776165636a23c2fb02c09770f2fec918a20d54e199` |
| `dist/index.html` | `9f03f109654e2691ce48de005b5553d1e2a973cad75ad068180a1465c0e2a16d` |
| `dist/assets/index-Bp9GOmBD.js` | `81e4dba270650dbeaeb0f97d12b35ab26bd13a6d03b5967bf5aa79868b4e5d28` |
| `dist/manifest.webmanifest` | `2eb7555a102641e6f3040d3c79e68ddb11578c6c01327e29ed6e1fb3a688940f` |
| `.openai/hosting.json` | `d03ed661cc899a2952c2e7b62a4cc7799c3463b60c8d81e304b1e709392551e5` |

## Source/build correspondence

The audit inventoried 57 build-input files: the 30 unique normalized paths declared by the current Rust `.d` dependency file (including embedded pixel/UI atlases and shaders), all game frontend source files, and the workspace/crate manifests, lockfiles, frontend config and build scripts. Their sorted absolute paths were normalized to workspace-relative forward-slash paths, then each relative path and its bytes were concatenated into SHA256, matching the PWA hash recipe. The resulting source snapshot fingerprint is **`3e2c7d51545e8af7bd798a6ad3682c157aae66a6adc1f64ceae775aaedf0e17d`**.

No Rust declared dependency is newer than the earliest generated normal WASM (19:05:44.574 UTC). None of the 57 inventoried inputs is newer than the bundled frontend build (19:05:48.377 UTC). This agrees with the completed build log and the inspected source snapshot. Timestamps/logs support correspondence; hashes identify exact source/output bytes. An independent compiler rebuild was outside this read-only review, so bit-reproducibility is not asserted.

| Relevant source / asset | SHA256 |
| --- | --- |
| `Cargo.lock` | `df228d9f83ef57321dcb768b79d948e9aa24456c0952e971fc8d6fcd7e806bdd` |
| `crates/game_core/src/lib.rs` | `e2ef687c9f14d8b75af6b36fbb220b8244979ff35a042d8b0ce6d579fabd6702` |
| `crates/game_web/src/boss_layout.rs` | `97ae0496ec27b89510b46ad21785a377d2ca6ae0d88d1fe95dcbc30a1758e353` |
| `crates/game_web/src/boss_overlay.rs` | `fd9ff6b24cf69d3c43144571815d071c251485b002cba8bcad27bced8b02810f` |
| `crates/game_web/src/browser.rs` | `2be9f952d91a3994688076f0c0a57fc468c254bd05073a6f14469156045143c5` |
| `crates/game_web/src/pixel_ui.rs` | `c66a24bf4c105df8fc5121c8c4ab8a797a19bd3eb51d5dfc067f2df68a7dd94a` |
| `assets/pixels/riveter-atlas.json` | `6bbf5a1333b486d54a7fb7f3600d89220dd774e1e80e9a699ad4edf75231247d` |
| `apps/game/src/main.ts` | `e0d8ee22777862a08558c4b034b20f24af98fa1ae25c3afc9e37df837c1c0cc7` |
| `apps/game/src/pixel-ui.ts` | `b62ef0f6e0ba55be1b506cfd03dae3cc4af10be1bff5c5b43bb3583c92ded3c3` |

The Riveter atlas is a declared Rust build input and predates the normal build; it is embedded in the game rather than separately fetched as a concept image. The production layout hash still matches the independently tested 42-case standalone cache prototype.

## Acceptance boundary

This audit accepts local packaging, fixture exclusion and separation. Root owns current visible WebGPU/WebGL captures, offline/recovery execution and any owner-authenticated private deployment check. Full natural Scout progression, all-fifteen browser choreography, physical-phone multitouch/readability/60 FPS/soak and physical PWA lifecycle remain pending until actual evidence exists. No release-wide or phone performance completion is implied by successful package parsing or hashes.
