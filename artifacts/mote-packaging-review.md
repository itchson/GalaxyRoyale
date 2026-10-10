# Mote normal-package release review

Audit recorded 2026-10-10T20:16:34.058Z. **Local normal package accepted for `galaxy-eaac3b23e213e59b`.** This reviewer changed only `work/mote-release-qa/` and performed read-only source/file/hash inspection plus actual WebAssembly binary parsing. No production/shared manifest/dashboard edits, broad test reruns, game/GPU execution, browser automation, publishing, credential access or authenticated remote execution were performed.

## Exact frozen target

The final normal build is root's `artifacts/mote-normal-build.log`, SHA256 `ac84d8744561ad6fafdda86acb3b02cb57209c4eea61f678c8366b89010b00aa`. It records both successful release builds, game TypeScript/Vite output, the eighteen-asset PWA and a separate studio build. Root confirmed build exit0 and source/package freeze. Earlier `be3ae02` / `galaxy-79774fbb09d2dfb9` publication and work-only Mote/death/cadence proofs are historical evidence, not this package or current browser execution.

The current cache name reproduces exactly by hashing every sorted packaged relative path and its bytes, excluding only `sw.js`. Its eighteen ASSETS equal the actual packaged files exactly, without duplicates or missing files. The standalone worker is additional to that asset count.

## Normal dual-WASM and exclusions

Both real binaries parse as `WebAssembly.Module` instances in Node: WebGPU has 48 exports and WebGL2 has 44. Both expose the actual normal step/profile/diagnostic/boss-UI APIs. Neither exports `checkpoint_contract`; that name is absent from their binary bytes, generated JS/types and bundled shell. Work-candidate directory names are absent too. Parsing inspects real binaries without creating a graphics device or executing a game.

All eight generated files match `apps/game/public/wasm` and `dist/wasm` byte-for-byte:

| Backend / file | Bytes | SHA256 |
| --- | --- | --- |
| webgpu / galaxy.js | 77645 | `c1f0fe72291fc2713b9165de3dffdc470355386b6f31caa7756db14447711187` |
| webgpu / galaxy.d.ts | 7320 | `c1a6f93bb491d2b5565d37d46e80a3fd9ee74eb9785f8c0b49d13c7d49b331fb` |
| webgpu / galaxy_bg.wasm | 2118609 | `f4ed153e57f5252ccb761fcb8823f218a843017dd0fb7b4f47a14bd49c4eb0b4` |
| webgpu / galaxy_bg.wasm.d.ts | 4630 | `f2b9f908eb1a4692d43f4fe9c510f24a115c8057d28e037b5898956dbe74b8b9` |
| webgl2 / galaxy.js | 80969 | `327185a19e7a9a959f377c77a5d99eb159c20295a55e5ed4845df6612380d123` |
| webgl2 / galaxy.d.ts | 6054 | `dcff0b67f31bf5f46ddc07ab386e5654c6654f5d14e585dcbe003680bc1db032` |
| webgl2 / galaxy_bg.wasm | 5765562 | `7ccfc8f16db8e5509897ad7e9812fd63d35e6a45f9148491e8c24b35b7f4c490` |
| webgl2 / galaxy_bg.wasm.d.ts | 3364 | `05ccf14d3ec3170f91aa338e7c23ba95e68d1b3f05cd79b5ee0ec1e878df0926` |

Normal WASM generated timestamps are WebGPU 2026-10-10T20:09:19.086Z and WebGL2 2026-10-10T20:09:21.470Z. The locked normal build selects only webgpu or webgl with no default features and excludes qa-contract. The normal release dependency list contains no cfg(test) death, Mote or cadence test module. Whitelisted diagnostic scenes remain intentionally shipped for frontend `?qa=1` use; compile-time contract/test fixture exclusion is distinct from removing the diagnostic API.

## Source composition and unchanged persistence

The independent `composition-check.mjs` reconstructs the accepted composition from the complete frozen Mote core and accepted shared early-dead helper. The actual merged tick equals the accepted helper ordering plus the Mote update call; the shared helper is unchanged. The entire core matches that expected composition after documented formatting/comment normalization, work-only absolute module-path removal, production cfg(test) helper-test declaration removal, and private radiation module declaration order adaptation. String, character and raw-literal contents remain exact. Production adapter/controller/art match the accepted final Mote files after only their production atlas include path and comments/whitespace are adapted.

This is source correspondence, not an independent bit-reproducible release compilation or new merged combat test. The actual Rust Profile fields are unchanged from the accepted candidate and default schema remains 3. Current strict TypeScript storage SHA256 is unchanged from the prior accepted release. Live Mote/controller state is ECS state; no persisted radiation/heat/DOT field or save migration is introduced. Root's recorded native/strict/build checks are separate execution evidence; the earlier independent eight-probe candidate test run is not relabeled as a merged production or browser run.

The current bundled frontend actually contains `biome-mote`, `LAST RECORDED EXPEDITION` and `NO EXPEDITION RECORDED`. Root owns visible death-report, Retry/Title, crafting and cue presentation checks. Static text inclusion does not prove their interaction/layout.

| Source | SHA256 |
| --- | --- |
| crates/game_core/src/lib.rs | `ec2b3c897d4b5dc1ec70891d383f19a008fc0425a5dedd691c298cb41ddd3cea` |
| crates/game_core/src/radiation_enemies.rs | `7991f3fd929230c8b6b424a746440251d18ddff589c2a3a996744ccd91168b6e` |
| crates/game_core/src/mote_controller.rs | `53134b719dd2fb361cc5feb7fbbc020a3b446458605f4096b175d55e64fe8109` |
| crates/game_core/src/mote_art.rs | `229916ac25331a0dc59add3a8c8a49274d2f4a3e01520ffa14a17d8b8965806f` |
| assets/pixels/mote-atlas.json | `004c24eefeccbe25d836dd8ad1ff00346f86879492c2bff3fe828721dc838714` |
| apps/game/src/main.ts | `386fbd1520ab3a9d32c14703cb9aded7d7008b9451102f503ebfb21dd84169c1` |
| apps/game/src/storage.ts | `2340133e825315fa445469e640ab75e47c3d1bdb34fa332b1c52e68459f463b6` |
| Cargo.lock | `df228d9f83ef57321dcb768b79d948e9aa24456c0952e971fc8d6fcd7e806bdd` |

## Source/build correspondence

There are 34 unique normalized inputs in the actual release Rust dependency list, including all three runtime Mote modules and `assets/pixels/mote-atlas.json`. It contains no work source or cfg(test) modules. The accepted original atlas hash is `004c24eefeccbe25d836dd8ad1ff00346f86879492c2bff3fe828721dc838714`.

This audit inventories 68 build inputs: those 34 declared Rust/assets inputs, 14 frontend source files, game HTML, 12 workspace/crate/config/lock/build files and 7 nongenerated public assets. Sorted workspace-relative paths plus file bytes produce SHA256 **`ef319b04076fabf4c5bef9c7e833e3125311962682159261a83951a98278f2e9`**. Generated public WASM is checked separately against dist. No declared Rust input is newer than the earliest generated normal WASM, and no inventoried input is newer than the bundled frontend. Hashes identify exact bytes; timestamps and the actual normal-build log support correspondence but do not establish bit-reproducible compilation.

## Game-only PWA and studio isolation

The cache contains game HTML, hashed shell JS/CSS, manifest, three icons, three font/license files and eight backend files. No studio, production/project JSON, work evidence, test fixtures, hosting configuration or credential files are packaged. The shell has no studio project API reference. The worker scopes same-origin GETs, excludes /api/ and /studio, caches only its actual listed files and retains explicit ACTIVATE_AT_DOCK activation. Offline execution, update behavior and device lifecycle require separate execution evidence.

The studio build remains separate from root `dist`. The inspected hosting binding selects static.directory=dist; no project identity, credential or endpoint is written in this review. That binding does not encode or prove an authentication policy, deployed bytes or owner-authenticated remote execution.

Exact current ASSETS:

- `/assets/index-DFoABeEK.js`
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

## Acceptance and pending gates

This accepts the identified local normal package, accepted source composition, unchanged save boundary, fixture exclusion, source correspondence and studio isolation. There is no observed blocking local packaging defect. Root owns current visible WebGPU/WebGL2 Mote/death/craft acceptance, normal offline/recovery checks and publication. Natural full Scout/campaign/boss progression, human balance/evasion/listening, physical-phone multitouch/readability/lifecycle/60 FPS/soak and authenticated current private remote execution remain separate pending gates unless independently evidenced. Previous candidate, hangar or death reports are not substituted for those current observations.

## Reproduce read-only audit

`node work/mote-release-qa/audit.mjs galaxy-eaac3b23e213e59b artifacts/mote-normal-build.log`

Run only against the named frozen dist/source target. The audit writes only its owned package-evidence.json. All exports, cache entries, file sizes/hashes, source fingerprints and source/composition checks are preserved there; `hashes.json` fingerprints this review and audit files.
