# Final adaptive HUD / facility-label source and normal package review

No concrete source or package blocker was found for `galaxy-cc8da99d023ae67a`. This review pins the final working inputs and actual normal package. Exact committed source and the official archive are a separate pending stage.

## Identity and reproducibility

- Normal JS: `index-B7Fh30JY.js`; CSS: `index-DJUrJ7jd.css`.
- WebGPU content directory: `3c47eae459be4db04e65eda321a1b3cfebbb20dcaa44002291852222254e2c5a`; actual module 2,605,114 bytes / 58 exports.
- WebGL2 content directory: `46109afebf38148945721d7340e90ecc55a0645e5d13d59e966b770877a157f0`; actual module 6,232,238 bytes / 54 exports.
- 151 source/build/validation inputs plus 20 package files were copied exactly: 171 captures, no changed input omitted. Each role is explicit in `source-package-pins.json`; test fixtures are not normal runtime payload.
- Raw SHA-256 pins actual working bytes. The additional text hash permits only CRLF/LF representation comparison with later committed text. Binary bytes are never normalized.

Owned read-only verification after freeze:

```powershell
python work/adaptive-label-final-release-review/verify.py --live
```

The inspector was executed against these exact normal assets. It compiled each actual WASM binary with Node `WebAssembly.Module`, parsed the actual ES2022 bundle and binding glue, checked both four-file directory digests, and confirmed public/dist correspondence. This is binary/module inspection, not browser GPU execution.

## Final label and presentation seams

The root `paintFacilities` projection uses the unshaken HUD camera, actual world viewport and the same logical UI scale. It checks the complete native 100x10 logical label rectangle against the safe viewport, exact docked status strip, actual header icon rectangles and touch pads. It chooses below (0), above (1), or hidden (2). The facility body hit remains at its original projected world center and diameter; label repair does not move the hit target.

The compiled bridge forwards each facility button's `data-pixel-value` with its measured rectangle and text. Attribute observation already includes that value. The native branch retains font 1 and the original below packet for value 0; above is centered at `rect.y - 12`; hidden emits no glyphs/transforms. Nonfinite values default to the original below state. The new native tests verify actual nontransparent corners, unlit material, constant letter scale, hidden output and footer clearance. The actual compiled bridge regression verifies all three values without changing the measured body rectangle.

CSS fallback labels consume the same above/hidden state. The earlier persistent graphics notice remains authoritative over ordinary expiring toasts during loss/preparation/failure; unavailable canvas visibility remains hidden until pixel-ready. The current bridge retains intersected root/scroll clipping and preserves full measured `status-hit` reservations rather than confusing the minimum physical hit with the seven-logical glyph slot.

The shared native flight geometry, paired `Float32Array` status/slot exports, actual unshaken attached-bar projection and Reduced Motion warning behavior remain consistent. Current `Hud.audio` is freshly supplied before flight/recovery audio resume. This audit does not claim to have listened to audio or executed the new browser layouts; root/Production browser observations are separate evidence.

Compared with immutable c707, the only non-generated existing inputs changed are main.ts, pixel-ui.ts, style.css, ui_scene.rs and flight-status-pixel.test.mjs; ui_facility_label_tests.rs and facility-layout.test.mjs were added. 135 common source-input paths remain byte-identical. The generated backend pairs/build map changed as expected. `source-seams.json` records the complete exact delta.

## Package, materials and checkpoint boundaries

Both normal binaries contain the exact current world and presentation WGSL source (four backend/source checks) and all eight actual embedded atlas JSON literals (16 backend/atlas checks). This proves packaged source correspondence, not GPU visual appearance. The renderer/material inputs retain the 16-byte instance, 32-byte transform and aligned 304-byte globals contracts, full backing-resolution world pass and separate logical UI pass. UI materials remain unlit; signed soft emission is separate from normal hard-object geometry. No arbitrary minimum hard-quad size was introduced by the label repair.

Current schema-4 ship/cargo/equipment sources, APIs and validation inputs are pinned. Normal bindings expose current craft/upgrade/equip, map/course, recovery and both presentation-bound methods. `checkpoint_contract` is absent from each actual normal binary, JS glue and declarations. The isolated contract fixtures are not added to the normal package.

The normal dist contains exactly 20 files, including SW; its cache list contains exactly 19 payloads. Cache derivation independently recomputes `galaxy-cc8da99d023ae67a`. Hashed entries in builds.json, the bundle's backend imports, the complete four-file directories and every precache path agree. HTML/manifest dependencies exist. Navigation uses the worker's own CACHE index, avoiding unrelated cached HTML. The game package excludes Studio/API payload; a separate Studio build log is not evidence that Studio was packaged with the game. The official staging step is expected to add only `dist/.openai/hosting.json`, making 21 archive files; that archive has not yet been supplied.

## Executed evidence and remaining gates

Root final logs copied into `root-validation/` report 406 native passes (37 engine + 3 external engine + 223 core + 112 web + 31 external layout), strict native and both WASM Clippy, strict TypeScript/normal dual build, and 240 shell cases: 239 pass / one Windows file-symlink capability skip / zero fail. Root's isolated headless WASM serializer/SaveStore run passed 18 cases; it uses fake-indexeddb and supplied inventory/history fixtures, and is not an earned-reward or browser proof. This specialist did not rerun builds or full suites.

Current root/Production browser captures and responsive results must retain their own actual dimensions/backends. Physical phones, notch/safe-area interactions, natural-play balance, human readability/listening and extended device soak remain outside this source/package review. The previous a9c3/c707/810 freezes remain historical and unchanged.

The next exact audit requires a full committed source SHA and named official workspace archive. `archive-stage/check.py` is ready for those inputs; it will compare reviewed source to Git blobs, generated pairs to current public/dist, and every tar file to this pinned package without extracting it or modifying source.
