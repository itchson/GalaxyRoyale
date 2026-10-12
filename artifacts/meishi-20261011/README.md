# MEISHI card and collections — actual local evidence

The final tested runtime is commit 69db89ccf5895390807383fcdd37c3413ba2dc68 on codex/testing. standalone-tested-build.json pins its source and package hashes. Later evidence/Studio-only commits require the normal cached rebuild before the final preview; the source hash must remain identical.

## Delivered behavior

MEISHI is a standalone read-only card with a native, corporation-trimmed frame, selected animated portrait, current RAICHO and effective loadout, retained RENKO/HUGO/PAN point balances and exactly eight anonymous-or-earned patch slots. Close is its only button. No collection links, tabs, Settings or save footer appear on the card. Escape/status icon open the live RAICHO menu; collections and MEISHI are separate entries there. Card frame and its children share one existing camera-independent UI group with local transforms. No new renderer, duplicate wallet or save schema.

Cargo shows carried/banked materials and active delivery items; named tier-one materials are subsets, not extra totals. Hull/ability collection filters expose only discovered records and use real Rust craft/equip/upgrade gates. In-place filters retain their visible scroll/focus. Advanced crafted-component recipes remain task-1cba1e46, not implemented here.

## Final verification

- Root final shell: 414 passed, one existing Windows file-symlink skip (415 total); TypeScript passes. Earlier negative logs remain and are explicitly superseded.
- Root game_web native library: 212 passed; strict workspace/all-target Clippy passes. Earlier whole-workspace baseline: 631 passed, one ignored cosmic export, before the later three native UI-only tests. Do not sum that into an unexecuted final whole-workspace run.
- Normal WebGPU/WebGL2 WASM + web/PWA + Studio build passes. Earlier 42 actual WASM/save-contract tests also pass; those serializer sources did not change in the card correction.
- Specialist native card fixture: all nine type/main combinations, all eight earned patches, worst 54,023/60,000 quads. This conservative old menu-plus-card fixture includes controls removed from the final card; it is native composition evidence, not a measured current DOM packet or GPU benchmark.
- New native surface: 14 bounded quads, existing clipping/transforms; two tests verify tiny/fractional/group geometry and atomic admission against both shared quotas.

## Browser observations (Codex IAB)

Controlled QA origin meishi-review.localhost, final runtime above. Existing meishi-check.localhost run was preserved. Actual console renderer commands confirm WebGPU and WebGL2 (standalone-renderer-*.txt); no warnings/errors were reported in the inspected browser log snapshots.

- standalone-webgpu-1280.png and standalone-webgl2-1280.png: full card, all eight slots, Close only.
- standalone-webgpu-848.png and standalone-webgl2-848.png: compact 848x390 landscape, all eight slots fit, no scroll required. DOM observation: only visible menu button is menu-close, card content 248px high.
- standalone-keyboard-640.png: short 640x280 window; Tab from Close enters labelled read-only card, End reaches its last patch row (214px viewport, 248px content, scrollTop33.6). Escape from the focused card returns to flight.
- PWA Apply update at Haven was used through actual UI. APR#B8AAA7F9CBAC3EE2 / PAN HIKARI / LV1 / XP6 / currency[0,0,1] / Flux DriveR1 / no patches remained unchanged after update and renderer reload. This pass did not award those pre-existing points/XP.
- cargo-webgpu-848.png: separate cargo, no duplicate point wallet. abilities-filter-fixed-webgpu-848.png: OWNED remains visible and selected, scrollTop100.8; focused row210.8..254.8 inside scroll viewport142..323.6. Blueprint hull filter reports no found matches and does not reveal hidden hull identities.
- Temporary QA tab closed and viewport override reset. One owned preview listener at4173 (same PID holds lease4172), one Studio API4174/frontend4175. No extra game service was launched.

## Historical/negative evidence

card-webgl2-*, card-webgpu-848, settings-webgl2-848 and initial/layout browser receipts predate the standalone correction and show the old menu. negative-card-single-column-1280.png shows the superseded tall card. abilities-webgpu-848.png is the superseded focus/scroll jump; abilities-filter-fixed-webgpu-848.png proves its correction. Earlier shell failures include a fake-DOM selector limitation, an accidentally relabelled live-status aria name (restored), and the old CSS breakpoint assertion (updated for the standalone layout). Retaining these is not a claim that they pass.

## Scope limits

Desktop IAB presentation and keyboard checks are complete for the views stated above. Physical-phone testing, broad browser/device coverage, human art/audio approval and 30-minute FPS/memory soak remain pending. No measured runtime speedup is claimed. PAN native256 dock files and reviews are an offline candidate, not a shipped dock replacement. Eight-player signaling has real HTTP evidence in the separate runtime-studio artifact, but playable WebRTC co-op remains pending. Specialist handoffs separate author tests from independent reviews.

The local Studio records actual assignments, decisions, review findings, final build evidence and these remaining gates. All manifest entries hash raw bytes; .gitattributes prevents line-ending conversion inside this archive.
