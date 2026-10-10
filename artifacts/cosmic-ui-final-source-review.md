# Final semantic menu source review

No blocking source defect was found. This bounded read-only review covers `main.ts`, `index.html`, `pixel-ui.ts` and the actual native semantic painter. Production, dist, API, prior handoffs and published evidence were not modified. Root owns complete release checks and browser execution; Production owns responsive and physical menu QA.

The six-page switch sets all panel hidden states, persistent `dataset.active`/`aria-selected` and roving tab indices from one selected page. The actual registered ArrowLeft/ArrowRight/Home/End handlers wrap the six pages, prevent browser scrolling, switch the page and focus its tab. Page switching scrolls the selected tab into view. The out-of-dock Upgrades route replaces purchase content with the truthful Haven hint. The exceptional victory Credits route shares Campaign's panel and leaves Campaign keyboard reachable; it is not an additional seventh navigation tab.

`menu-wiring.test.mjs` uses the installed TypeScript parser to extract and execute the actual declaration, switch function and event registration from the pinned `main.ts`. Four checks passed. The existing actual `PixelUiBridge` test file also passed all six checks, covering persistent selection versus transient focus, category kinds, earned emblem state, fractional icon fit, finite range progress and both-axis clipping. These run against a small semantic DOM harness; they are not real browser layout/input proof.

Root/nearest page scroll bounds intersect horizontally and vertically before node submission. Hidden pages and wholly clipped nodes are skipped. Scroll/focus/input/class/hidden/selection changes invalidate the bridge, which synchronizes before actual rendering. Native semantic buttons separately clip their fitted label to an inner padded rectangle; persistent selected navigation has a contrasting fill and one bottom bar independent of another button's hover. Settings reserve the checkbox gutter or bottom slider strip. Group placement uses physical DOM coordinates divided by the same logical UI scale; native scene projection remains independent of world zoom. No control or font implementation was changed in this review.

The reported14px DOM font at UI scale1 correctly produces bridge font1.75, which the authored RoyaleMono painter rounds to2. This changes advance relative to the DOM font; it is not itself evidence of clipping. `metric-glyphs.rs` calls actual current native `ui_scene::draw` for27 recorded intrinsic Settings rectangles: the settled `f1e-settings-720x360`, `f1e-settings-wide-1280x720` and `95d0-settings-portrait-360x800` measurements from Production's separate desktop session. The owned `metric-fixtures.json` preserves exact widths/heights and bridge font values1.75/1.125. All nine setting texts in each case retain every expected authored glyph cell, with all cell corners inside their native text/control gutters. Labels are relocated into a fully visible group for this intrinsic-fit probe; actual scrolling/visibility is deliberately excluded. These historical/final recorded DOM extents do not constitute a new browser reproduction for this reviewer.

Known bounds remain explicit: text wraps into the supplied rows, buttons may shrink or ellipsize long labels, unsupported authored font glyphs remain empty, and cell clipping uses occupied centers while solid semantic rectangles are geometrically clipped. This review establishes no new universal long-prose or partial-scroll readability claim. Actual tiny/human/physical observations remain with Root/Production.

Executed:

```powershell
node --test work/cosmic-ui-final-source-review/menu-wiring.test.mjs
node --test scripts/test/pixel-ui.test.mjs
rustc --edition=2024 work/cosmic-ui-final-source-review/metric-glyphs.rs -L dependency=target/debug/deps --extern engine_core=target/debug/deps/libengine_core-c6e44304da45d14c.rlib --extern game_core=target/debug/deps/libgame_core-684f3d0a8e530f28.rlib --extern game_web=target/debug/deps/libgame_web.rlib --extern serde=target/debug/deps/libserde-87e9527ea0c869bc.rlib --extern serde_json=target/debug/deps/libserde_json-b89616ee5c7ce7b6.rlib -o work/cosmic-ui-final-source-review/metric-glyphs.exe
& work/cosmic-ui-final-source-review/metric-glyphs.exe
```

Fingerprints and owned evidence are in `evidence.json`. Reviewed production hashes were unchanged at final readback. Root separately reports282 Rust tests,89 shell tests, native/both-WASM Clippy and normal dual build cache95d0a4d39116971e; this specialist does not relabel those checks as independently rerun or browser/device acceptance.
