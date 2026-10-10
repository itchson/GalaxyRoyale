# Scar / Mote audio / champion emblem normal-release review

The inspected local normal package is accepted within this review's scope: source correspondence, actual WASM parsing, output hashes, dependency/cache isolation and declared QA warp policy. No remaining blocking source or packaging defect was found. This review does not execute the browser, hear the audio, play a natural campaign, inspect a physical phone or authenticate to a private deployment.

## Exact package and baseline

`package-evidence.json` was captured at 2026-10-10T20:54:36.951Z from cache **galaxy-86ea0d1b0d17f148**, with exactly 18 game assets. Its sorted path/content digest independently reproduces the PWA cache identity. Every PWA asset corresponds to an actual file, with no extra or duplicate entry. The worker excludes API/Studio requests and activates waiting updates only through the existing `ACTIVATE_AT_DOCK` message.

- WebGPU WASM: 2,243,181 bytes; SHA256 `08ea410749efe985d0ca85b927252ff4bf9cf35d23614180a32c21a08c88813b`; generated 2026-10-10T20:52:16.287Z.
- WebGL2 WASM: 5,870,762 bytes; SHA256 `82711ff7ba488c749bb4c4ed757ac19e3e929ddcb0dd589cc1702fd84c779cef`; generated 2026-10-10T20:52:18.508Z.
- Both actual binaries parsed using `WebAssembly.Module`; WebGPU has 48 exports and WebGL2 has 44. Normal profile/step/diagnostic/boss-bounds exports are present. `checkpoint_contract` is absent from their actual export tables and generated bindings.
- All eight public/backend generated files equal their dist counterparts byte-for-byte. Studio is built in its separate output directory and is absent from game dist/cache. Work candidates, test fixture modules and production-dashboard endpoints are absent from normal game outputs.
- The actual normal compiler dependency trace has 38 production Rust/asset inputs; it excludes death/warp/controller/audio-event/region/cadence test modules and all work paths. Scar and champion emblem sources/atlases are present as compile-time dependencies, so they add no standalone fetched atlas/preview asset.
- The total 72 source/config/public inputs have fingerprint `d4c40129b7b1efda4cceb3577a9731636279c4447f5759f64e93bb8422d14b5d`. No declared Rust input is newer than the earliest generated WASM; no inventoried build input is newer than the bundled frontend. The full normal build log is `artifacts/scar-audio-normal-build.log` and includes both release builds and the exact cache.

The prior deployed runtime/source `573906769122140e2e54021bdaf037ad6f887673` and prior cache `galaxy-eaac3b23e213e59b` are historical baselines. Metadata HEAD `51b7bc5` remains separate from the uncommitted integrated working source captured here. A later Git/source or deployment identity must be recorded by the producer; this audit does not invent one from the current package hash. `.openai/hosting.json` specifies static directory `dist`, without establishing remote bytes, access or owner-authenticated execution.

## Runtime and art correspondence

The read-only `composition-check.mjs` verifies actual fixed-step tick, shared early/final death helper, purchase and dock-reset methods against the pinned preintegration runtime. Persisted Rust Profile fields and the strict browser schema remain unchanged (schema 3; storage SHA256 `2340133e825315fa445469e640ab75e47c3d1bdb34fa332b1c52e68459f463b6`). Cadence residuals, ship XP/rank gates, cargo/crafting rules, ordinary boss progression and natural death semantics are not changed by the art/audio hooks.

The production Mote host preserves the accepted actual Charge-entry/real-spawn event rules, reservations/order and partial pulse intensity. The existing controller retains the accepted charge/pulse/recovery timings. The SFX source retains the exact accepted tones/noise, separate buses, resource caps and lifecycle. The region adapter matches the independently reviewed unencoded origin, interpolated camera, already-eased world extent, frozen cosmetic time and single shared remaining charge. A rejected whole Scar does not restore the old planet/rings. Earlier background layers can still consume the budget and omit a whole motif.

Scar/emblem atlas bytes and authored helper semantics match the accepted candidates after declared include-path/format adaptations. `provenance-correspondence.json` records exact provenance and the authoring generators' only adaptation: their output provenance basenames. No AI/model/texture import, new runtime image load, hazard, save field or reward was introduced by this art.

Champion UI uses authoritative `Champion.patch` and `tournament_won` for earned state. Hover/focus no longer lifts an unearned patch. The ninth emblem is presentation of existing victory, rather than a ninth persistent champion patch. The bridge recovers only authored 24/32 extents within 1/64 logical unit and preserves the measured center; it does not resize materially undersized UI. The native helper retains caller pixel/work budgets, cell-center clipping and separate UI projection. Existing fractional-edge half-cell bleed and actual device readability remain visual limits.

## QA warp finding and resolution

Initial source review found that the new WARP path called the real dock reset before isolation: carried materials bank, an active run is recorded, courier delivery can advance, and an active natural tournament duel is forfeited. A ready site quest could also still grant rewards after warp. The fresh-profile equality test did not cover those existing in-flight transitions.

The producer resolved the reward gap with an immediate `boss_qa_scene` guard in `interact_site`, before exploration or mutation. The retained real dock-reset policy is now disclosed by console text: QA combat/site rewards are disabled; the current run is banked and an active duel forfeited. Four actual-core warp regressions cover finite seeded placement, malformed-input no-reset, blocked ready-quest claim with ordinary once-only claim preserved, and existing run/material banking exactly once. Their run/forfeit semantics reuse the unchanged dock/reset path; no fake run, reward, timestamp or saved field was added. Both browser and core parse guards reject nonfinite/out-of-range coordinates; browser command availability is behind the existing explicit QA mode.

This is intentionally a QA scene transition using the current save, not a separate disposable save. The report does not claim that WARP preserves every profile byte for an in-flight expedition. Its reward suppression is distinct from the truthful pre-existing dock bookkeeping.

## Evidence attribution and correction

The producer's inspected logs record 242 native tests (34+3 engine, 125 core, 49+31 web), 79 shell tests and strict native Clippy. The normal build completed; this reviewer did not rerun broad tests. The earlier immutable work-only composition archive records this reviewer's separate 30 selected native tests, seven recorded-WebAudio API tests and 1,800 actual normal-native comparison ticks. Those historical results are not promoted to execution of this newer normal browser package or its separate WARP/emblem integration.

Correction to the frozen composition README: the full seed-17 fixture appends 4,854 cells at all three tested zooms. Actual `region-calls.json` shows remaining **5200→346 at .6/1.03**, and **5612→758 at 1.6**. The original README's all-zooms 5200→346 caption was too broad; the data and budget invariants were correct. The previous archive and manifest remain unchanged.

Actual moving/crowded threat readability, browser DSP/listening, full natural Scout/champion/tournament progression, mobile multitouch/device performance, PWA device lifecycle/soak and owner-authenticated private remote execution remain separate pending gates. Representative screenshots or sampled desktop FPS must not close those gates. Fresh browser observations, if made by the producer, require their own package identity and evidence rather than being inferred here.

Reproduce the read-only package audit with `node work/scar-audio-release-qa/audit.mjs galaxy-86ea0d1b0d17f148 artifacts/scar-audio-normal-build.log`. It writes only this work directory and fails if current assets or source correspondence no longer match the declared target.
