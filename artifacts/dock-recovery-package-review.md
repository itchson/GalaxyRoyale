# Dock, LOD and renderer recovery package review

The frozen normal package passes this independent source/package inspection. No blocking identity, export, facility-position or PWA payload defect was found. This review did not rebuild the package, interact with the browser or mutate production source/dist. Exact committed-source/archive correspondence remains a separate gate after the root packaging handoff.

## Observed package

- Cache **galaxy-a8d43618fa2b35c0** independently recomputes from the sorted pathname/content bytes of all 18 cached assets. The service worker and manifest are included in 19 served files; the worker itself is not precached.
- Actual WebAssembly binary parsing gives **51 WebGPU exports / 47 WebGL2 exports**. Both expose `galaxy_dock_facility_at`, `galaxy_install_recovery` and independent `prepare_renderer_recovery`; the former asynchronous exclusive `galaxy_recover` is absent. The opaque recovery free export is present.
- No `checkpoint_contract` or inspected compile-gated QA/test exports are present. The normal public diagnostic/device-loss controls intentionally remain; this is not a claim that debug controls were removed. The normal builder uses locked release builds with only `webgpu` or `webgl` features and excludes `qa-contract`.
- Both sets of WASM/JavaScript/type declarations are byte-identical to their generated public copies. Fonts, license, icons and manifest also match their public source bytes. HTML asset references and manifest icon references resolve within the package.
- The game-only payload contains no Studio, production records, work evidence or API files. Studio has a separate build destination. PWA fetch handling remains same-origin GET only, excludes `/api/` and `/studio`, offers an offline navigation fallback and activates updates only after `ACTIVATE_AT_DOCK`. This establishes file/worker contracts, not browser offline-install acceptance.

| Binary | SHA-256 |
| --- | --- |
| WebGPU | `3bb4363964b90ec109363dc2eee992f1d779d3ef3013eeedb0e46b040368d317` |
| WebGL2 | `8867d2bf8c5322217abb6e60ada7775ee2381d841134eee51ed594d06046df97` |

## Source integration

The review pins **87 runtime/build inputs plus 7 public static inputs**. It includes the actual pnpm lock/workspace files, all three Rust crate source trees/manifests, source atlases, game shell, renderer shader and build scripts. The captured HEAD before the pending runtime commit is `c614c93e8b501233cf267f18d220bb75cc16c76c`.

Rust's authoritative facility centers are exactly `[-120,-80]`, `[120,-80]`, `[-120,95]`, `[120,95]`, matching the semantic shell positions and page order upgrades/hangar/journal/campaign. The WASM method delegates to that single core query; world presses do not use a second browser hit-test geometry. Core availability requires the actual player inside Haven's 240-unit radius and a finite point within a facility's 24-unit hit circle. All rendered building/pad corners fit that circle. Four visible groups emit 1,247 occupied quads and eight local-object transforms; Reduced Motion forwards frozen cosmetic time.

The new input interaction runs only on an initial left/touch press before mouse move capture. A successful interaction returns without a destination; consumed held-left drags remain consumed. Right mouse retains aim/fire and the shell passes no weapon-menu callback. Semantic facility buttons query the same core ID at the authoritative center and open the corresponding menu. Their accessible browser rectangles may be larger than the 24-unit world circle; they are explicit menu targets, not replacement world collision geometry.

The only current `Game` lib diff against the captured HEAD is module declaration, readonly facility query and render extraction hook. The serialized Profile struct remains identical with schema 3. Campaign, bosses, Riveter/Mote controllers and audio are unchanged against HEAD after LF/CRLF normalization. Storage's diff adds a readonly `persistent` getter; schema/currency/XP/campaign/CAS fields are unchanged.

Recovery preparation is a free asynchronous function with no live `Galaxy`/`Game` borrow. Synchronous installation consumes a candidate, validates its canvas, resizes and updates visible extent without replacing Profile/entities/accumulator/UI/zoom. The shell gates old-renderer drawing/resizing, applies the latest dimensions after install, and rechecks launch, modal, encounter, visibility and portrait intent before resuming. Real WebGL lost/restored listeners and errors are bundled. These are source contracts; device-loss attribution, real restoration and lifecycle behavior are root-owned browser gates.

Reduced planets/rings use cached same-art 2× LOD local cells and complete-group admission. Current core test sources cover exact budgets, no orphan rings, retained nova, seeded identities, frozen time/materials and real corners. This review does not reuse the historical 180-case work-study statistic as current production acceptance.

## Evidence and limits

`work/dock-recovery-package-review/source-pins.json` records source/static SHA-256s and unchanged progression boundaries; `package-evidence.json` records every package byte hash and actual binary export name. The checker only writes that owned work directory. Root-owned logs are pinned: normal dual-backend build exit 0, strict native Clippy exit 0, and native test groups 34 + 3 + 152 + 81 + 31 = **301 passing tests**. These logs were inspected, not rerun or represented as independent browser evidence. The earlier focused facility/input handoff additionally records four new core and seven new shell tests.

Root reported actual browser facility/forge/pause observations; this reviewer did not replay those actions. Browser DSP/hearing, graphics raster quality, true context/device restoration, moving threat contrast, physical Android/iOS and sustained thermal/soak testing remain distinct. Matching copies, source pins and the build log establish traceability; pins alone cannot independently prove compiler provenance without a rebuild, which was explicitly out of scope.

Package acceptance here is complete; commit/archive acceptance is pending. The owned evidence is frozen before that later pass, which will be recorded separately rather than rewriting this report.
