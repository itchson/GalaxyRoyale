# Progression local packaging review

Audit recorded 2026-10-10T19:39:00.648Z. **Local package accepted.** Read-only inspection of the final normal package after the scoped dock pressed-state correction. No production edits, publishing, credentials, authenticated remote execution, GPU execution or broad test reruns were performed by this reviewer.

## Final target

Final cache is **`galaxy-79774fbb09d2dfb9`**, reproduced exactly by hashing sorted relative paths plus all file bytes except sw.js. The cache contains exactly **18 game assets**, equal to the actual packaged files. Intermediate `galaxy-c32242004fa4fe8a` and published HUD/Riveter baseline `galaxy-f37b303ee2258928` are not this final acceptance target.

`artifacts/progression-build.log` records the successful locked normal dual-WASM, TypeScript, game/PWA and separate studio build. `artifacts/progression-web-build-final.log` records the later frontend/PWA rebuild for the dock CSS correction. Normal WASM hashes stayed unchanged across that frontend-only rebuild. Log hashes:

- `artifacts/progression-build.log`: `d85a223066de79cfa268bd381a3cfd47959e20951c211963058124562d66a74f`.
- `artifacts/progression-web-build-final.log`: `138766192daabd2d14f8b7999291ec650bd246932de0d6b92529ef51584a6e86`.

Frontend bundled at 2026-10-10T19:36:31.308Z; generated normal WASM: webgpu: 2026-10-10T19:34:59.217Z; webgl2: 2026-10-10T19:35:02.182Z.

## Normal outputs and fixture exclusion

Both binaries parse as actual WebAssembly.Module instances in Node. WebGPU has 48 exports and WebGL2 has 44; both export the normal `galaxy_boss_ui_bounds`. Neither exports `checkpoint_contract`, and the fixture name is absent from both binary bytes, generated JavaScript/type declarations and bundled shell. Module parsing/export inspection does not initialize a GPU or execute the game. All eight generated backend files match public/dist byte-for-byte.

| Backend / file | Bytes | SHA256 (public and dist identical) |
| --- | --- | --- |
| webgpu / galaxy.d.ts | 7320 | `c1a6f93bb491d2b5565d37d46e80a3fd9ee74eb9785f8c0b49d13c7d49b331fb` |
| webgpu / galaxy.js | 77645 | `c1f0fe72291fc2713b9165de3dffdc470355386b6f31caa7756db14447711187` |
| webgpu / galaxy_bg.wasm | 2049157 | `a9305ad24848e7bc0aea9fcdbd18a4eddc07e338e5830b4ad72917fba175a291` |
| webgpu / galaxy_bg.wasm.d.ts | 4630 | `f2b9f908eb1a4692d43f4fe9c510f24a115c8057d28e037b5898956dbe74b8b9` |
| webgl2 / galaxy.d.ts | 6054 | `dcff0b67f31bf5f46ddc07ab386e5654c6654f5d14e585dcbe003680bc1db032` |
| webgl2 / galaxy.js | 80969 | `327185a19e7a9a959f377c77a5d99eb159c20295a55e5ed4845df6612380d123` |
| webgl2 / galaxy_bg.wasm | 5696274 | `430e9b9ae5c7d50e27d88892bb6d147ec8f75424bcb09ec05c98852b5630bbc2` |
| webgl2 / galaxy_bg.wasm.d.ts | 3364 | `05ccf14d3ec3170f91aa338e7c23ba95e68d1b3f05cd79b5ee0ec1e878df0926` |

The build script uses locked release builds, no default features, and only webgpu or webgl; qa-contract is excluded. The cfg(test) production cadence module is absent from the release dependency list. Work-only candidate libraries, tests/results and future radiation prototype are excluded from the package.

## Scope and isolation

The 18 cache assets are game HTML, hashed JS/CSS, manifest, three icon files, three font/license files and eight WASM backend files. No studio, production/project JSON, work evidence, QA fixture, hosting metadata or credential file is packaged. The bundled shell has no studio project endpoint reference. The service worker excludes /api/ and /studio, caches only listed same-origin assets and retains explicit ACTIVATE_AT_DOCK activation. Current offline execution, service-worker update behavior and physical PWA lifecycle require separate execution evidence.

Studio remains independent at apps/studio/dist; game output is root dist. The inspected hosting binding selects static.directory=dist. It does not encode an authentication policy. Local byte hashes and a hosting binding do not prove owner-authenticated private remote access, current deployed bytes or remote execution.

## Source/build correspondence

The audit fingerprints 64 inventoried game build inputs: 30 unique normalized paths declared by the current release Rust .d dependency list, 14 frontend source files, 13 workspace/crate/build/config/lock files and 7 nongenerated public assets. Sorted workspace-relative paths and file bytes produce SHA256 **`59cd5b9f3cda2b1d1153551e5d0f971aec48eecbc4e875d43f2a91745231a9ff`**. Generated public WASM is checked separately against dist. No declared Rust input is newer than the earliest generated normal WASM; no inventoried input is newer than the final bundled frontend. Timestamps/logs support source correspondence, while fingerprints identify bytes; independent bit-reproducible compilation was not performed.

Current integrated runtime core matches the independently accepted frozen cadence candidate exactly after LF normalization and removal of only work-specific path/test declarations and production cfg(test) test-module declaration. The accepted candidate is f038546fb7b630ccff18bf2b15ce6e2f5d973d2c9283dbec27fae62aa299e5e9. Its independent actual public-Game proof is recorded in `work/weapon-cadence-qa-review.md`, `work/weapon-cadence-qa/results.json` and `work/weapon-cadence-qa/hashes.json`. This package audit does not reclassify native diagnostic probes as natural progression.

| Source | SHA256 |
| --- | --- |
| `crates/game_core/src/lib.rs` | `0a574a35c9d23f955fff05246da2cf4bf9a8065f5db58c63c365124f982337e6` |
| `crates/game_core/src/campaign.rs` | `ce3701246d366042841cca3a61ce1c83ea15bbfcd312a05fe5d6d7f991d0b5bc` |
| `crates/game_core/src/campaign_runtime.rs` | `beaa27a19af8c62a23df0b6d6f820e924af51ed40f1d1e78a5cac25f8d7a4e58` |
| `apps/game/src/main.ts` | `959644e093a58c8602d8275f214d809433275ceb6ed89ad353b1de15b8f2657e` |
| `apps/game/src/style.css` | `c459520b8b65d179b193c1fe4961166a57a7083e0aff88c2df6333733bc6741c` |
| `apps/game/src/storage.ts` | `2340133e825315fa445469e640ab75e47c3d1bdb34fa332b1c52e68459f463b6` |
| `Cargo.lock` | `df228d9f83ef57321dcb768b79d948e9aa24456c0952e971fc8d6fcd7e806bdd` |
| `scripts/build-wasm.mjs` | `decc2c498fafdc38167b9e5a106a1b9c51de9392cf4d76394a5a24a7cb14959b` |

## Bounded UI source review

The added .dock-prompt:active rule has higher specificity than generic button:active and composes translateX(-50%) with the existing two-pixel pressed offset. It preserves centering instead of replacing the entire transform and moving the hitbox half its width mid-click. The scoped fix does not change unrelated controls. Source includes MAX LEVEL at ship level 100 and keeps the actual next-level threshold below 100; naturally earning that XP and browser interaction are separate checks owned by root.

## Acceptance boundary

This accepts local packaging, normal fixture exclusion, source correspondence and studio separation. Root owns the final visible WebGPU/WebGL reload/click/caption checks and any subsequent deployment. Natural early Scout progression, all-boss/full-campaign play, human balance/listening, physical-phone multitouch/readability/60 FPS/lifecycle/soak and owner-authenticated current private remote execution remain separate pending gates until actual evidence exists.

## Audit checks

| Check | Observed |
| --- | --- |
| Cache matches computed bytes | true |
| Assets exactly equal package | true |
| Forbidden package paths | 0 |
| Public/dist WASM files identical | true |
| QA fixture absent | true |
| Normal backend feature selection | true |
| Stale Rust inputs | [] |
| Stale final build inputs | [] |
| Integrated runtime matches accepted candidate | true |
| Scoped dock CSS composition present | true |
| Hosting selects game dist | true |

Exact game cache inventory:

- `/assets/index-DtOYte7_.js`
- `/assets/index-bpgry9H8.css`
- `/fonts/OFL.txt`
- `/fonts/RoyaleMono.ttf`
- `/fonts/Tiny5.ttf`
- `/icon-192.png`
- `/icon-512.png`
- `/icon.svg`
- `/index.html`
- `/manifest.webmanifest`
- `/wasm/webgl2/galaxy.d.ts`
- `/wasm/webgl2/galaxy.js`
- `/wasm/webgl2/galaxy_bg.wasm`
- `/wasm/webgl2/galaxy_bg.wasm.d.ts`
- `/wasm/webgpu/galaxy.d.ts`
- `/wasm/webgpu/galaxy.js`
- `/wasm/webgpu/galaxy_bg.wasm`
- `/wasm/webgpu/galaxy_bg.wasm.d.ts`
