# Settings and authored pixel controls

Implementation commit: `e88e78eade274d81f2cbf519738f3b7722d2f961` on `codex/settings-pixel-layout`, based on Root `6152611ec88653946f809f200a24a0de482cf265`. Tracked checkout clean after commit. Codex authored these changes; this is an implementation handoff for Root review, not an independent approval of my own source.

Seven owned production/test files: `apps/game/index.html`, `apps/game/src/style.css`, `crates/game_web/src/ui_scene.rs`, `ui_settings_tests.rs`, `ui_icon_tests.rs`, `ui_button_tests.rs`, `ui_map_tests.rs`. The two latter existing test files only update setting geometry and the map icon footprint/landmark expectation; map rendering/gameplay is unchanged. No main, bridge, shader, game simulation, saves, shared manifests, build output or services changed.

## Result and integration contract

- Settings use Audio, Display & motion, and Session sections. Wide layouts have two columns, compact logical widths use one column, and the existing menu-page owns scrolling. Label padding/gaps and headings are independent from the input strip. Existing nine input IDs and three action IDs remain intact.
- Every setting name is a `.setting-name` span. `UiNode.label_rect` / `control_rect` are optional `[x,y,w,h]` in group-local UI units; missing/null remains supported. Root's `a41196dff297426143d1415a4064caaf3faf8068` bridge supplies these exact fields. Native invalid, escaped or overlapping regions fail closed. At most 1/64 unit of measurement roundoff is recovered by intersection.
- Native captions select the largest complete RoyaleMono wrapped layout that fits. “Reduced cosmic detail” is 21 characters and 125 units wide at 1×. The new actual-packet regression checks every glyph's occupied cells for 130-unit single-line, 86-unit two-line and 50-unit three-line label rectangles. This does not guarantee every arbitrarily tiny DOM box fits: inputs smaller than a glyph remain bounded/fail closed.
- Native slider thumb is 6×10 logical units, with center travel between half-thumb gutters of the measured input. CSS WebKit/Firefox appearance/thumb rules match those dimensions. Input strip retains `--ui-hit` (at least 44 physical pixels). Checkbox label cards retain at least 44 pixels; visible checkbox geometry comes from the actual checkbox BCR.
- Eleven original 24×24 cached control bodies replace the earlier coarse upsampled symbols: settings gear, resume, restart, Haven/title, power/quit, telemetry/status, map/chart, close, saved/pending/error. Direct authored opaque cells include silhouette holes and four/five material shades. Artwork is centered within independent semantic hitboxes; no generic icon frame is added. Disabled, hover, save state, whole-cell clip and bounded admission semantics are retained. No emission/lighting or time animation was added.
- Facility CSS fallback now has transparent idle hitboxes and hidden labels, revealed on hover or focus-visible. Hidden-position labels still display:none. Root's corresponding native hover/focus-only bridge is required and was independently checked.
- Root's new ampersand glyph in `pixel_ui.rs` is needed by “Display & motion”; this branch deliberately leaves Engine-owned font source untouched.

## Executed checks

- `cargo test -p game_web ui_scene --locked`: 58 passed, 0 failed; 66 other unit tests and 31 integration tests filtered. Includes six new settings checks and retained icon/clip/reflected-parent/far-coordinate/budget tests. Final post-format log: `native-ui-tests.log`.
- `cargo clippy -p game_web --all-targets --locked -- -D warnings`: passed. This type/lint-checks all native targets; it does not execute the filtered tests.
- Strict `pnpm exec tsc --noEmit`: passed on this isolated checkout.
- Seventeen existing flight-menu, facility-layout and pixel-ui shell tests: passed. These use declared DOM/VM fixtures; they are not browser layout measurements.
- The exact two files from Root `a41196d` were copied into the owned review folder and its ten bridge tests passed independently. `root-bridge-review/pins.json` records exact source hashes. Read-only review confirms input disabled state, group-local measured regions, legacy label fallback and facility hover/focus gates match this native API. No concrete bridge defect found in this scope.
- HTML parser: all document IDs unique and all nine settings inputs retain one `.setting-name`. Prettier, Rustfmt and `git diff --check` passed.

## Actual native art evidence

`author_icons.py` is original authoring code; no model, image generation, copied art or runtime raster asset. Final-resolution rows are compiled into PixelBody. `authored-contact-sheet.png` is an authoring-only preview.

`packet_preview.rs` links the actual native engine/core dependencies and imports actual `ui_scene`, `pixel_ui` and `champion_patches` sources unchanged. It extracts all eleven controls at native 24-unit scale and at a 44px hitbox / 2.4 UI scale. `native-packets.json` contains the real engine cell corners/colors. `preview_packets.py` renders only those captured packets into `native-contact-sheet.png`, using a half-open software rectangle raster. This is not a browser screenshot or GPU image.

Actual two-row extraction: 6,302 submitted quads, 22 transforms, 1 batch. Each cached body has 164–422 occupied cells; the seven menu icons total 1,653 and all eleven controls total 3,151. Transparent silhouette cells are absent; clipped occupied cells remain charged by the existing budget. Pooled transforms and local geometry remain unchanged under the retained reflected/sheared and far-camera proofs.

## Pending acceptance

Root will integrate this commit with the measured bridge, Engine's font/minimap and current main. Normal dual-backend build, real 1280×720 / 848×480 settings scrolling/text, pointer thumb endpoint mapping, keyboard Arrow/Home/End interaction, touch gestures, tiny aspect layouts and user readability remain pending. No server/browser/push/merge or public/dist rebuild was performed here. Font/packet proof does not establish physical-phone usability or every subpixel icon cell being perceptually visible.
