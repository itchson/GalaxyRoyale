# Detailed KIKAI portrait correction

This revision corrects DORO and TALO torso silhouettes and replaces all three runtime portraits with 192×256 authored cell packs derived from pinned high-resolution references. DORA keeps its feminine anatomy. Native body, eye and mouth layers preserve independent manufacturer hair and crystal colors, expressions, fractional transforms and clipping. The UI now rasterizes at its physical presentation size while retaining logical layout and input coordinates; world raster behavior is unchanged.

## Evidence history

- `independent-review.md`, `source-pins.json`, `raw-cell-evidence.json` and `compact-equivalence.json` describe the initial source/art review. `metadata-review.md` records the subsequent canonical-LF correction. These are historical reviews with their own pinned inputs, not final browser acceptance.
- `native-536.txt` and `clippy.txt`: Root's native workspace tests and strict native Clippy on integrated 5bc8bff with the original detailed atlas 03b2651c77f6164c9261b880e407ba333318f67e4e3b1155248edca033e0db95.
- `shell-negative-fixture.txt`: an obsolete shell adapter lacked `dialogue.close`; this one test failed. Fixture commit 8479398 corrected the adapter. `shell-346.txt` records 346 passes, zero failures and the existing Windows file-symlink privilege skip.
- `build-before-mask-fix.txt`: successful normal WebGPU/WebGL2, TypeScript, game PWA and studio build on 5bc8bff. This is the initial browser candidate, not the final mask fix.
- `doro-webgpu-1280.jpg` and `talo-webgpu-1280.jpg` are deliberately retained **negative** browser captures. Their detailed anatomy is visible, but the old crystal color mask also tinted forehead skin. The initial source review did not catch that defect. Final captures have a `-final` suffix and supersede them only for their declared scenes.

## Limits

No source asset can display every source cell when its final screen footprint contains fewer physical pixels. This change preserves the source cells and removes avoidable UI downsampling; it does not claim infinite display resolution. The shared 60,000 UI-quad budget remains bounded and portraits are submitted atomically.

This is a portrait and shared-UI rendering correction, not a claim that every world asset has been reauthored. Human approval of the overall art direction, cold-start/peak-memory measurements, physical-phone validation, long performance soaks and full naturally earned training remain separate gates. The private hosted preview is not republished by this local validation.

## Final repair and browser checks

Art fix 1b5e408 was integrated as 8d9c29a; independent native regression a609fbf was integrated as 50b710d. Final atlas SHA256 is `31fa6a1b04d83f3d5fad377562e73c6ad35530b028626a1136c5b56afd9ea661`. Each crystal now has an authored region and connected saturated-facet mask, with facial skin excluded. Every sprite cell, palette entry and hair mask is unchanged by this repair. The independent addendum is `role-repair-review.md`; its old/fixed native logs preserve the real failing and passing controls.

Root rebuilt the normal packages at 8d9c29a (`capture-build.json`, `build-final.txt`) and captured all `-final.jpg` images from that package on isolated `kikai-final.localhost`, without overwriting the player's previous save. The follow-up 50b710d changes tests only. The renderer names were read through the actual in-game `renderer` command: WEBGPU and WEBGL2, each 640×360 logical pixels at the final 1280×720 viewport. The backend screenshots preserve those responses; a requested query-string backend alone was not treated as proof.

- WebGPU creator: DORO/HUGO/PAN, TALO/RENKO/HUGO and DORA/PAN/HUGO at 1280×720. Former forehead tint is absent. DORO and TALO have flat mechanical chest armor; DORA retains its feminine silhouette. These are scoped visual judgments, not automated anatomy certification.
- Creator controls tested at 848×390 and 320×180. Lower options scroll into view and Issue License remains reachable. The smallest layout scrolls the portrait and explanatory text out of view; it cannot show the complete form simultaneously or preserve all192 source columns at that physical size.
- Issued isolated DORO-HUGO-PAN license `B0894EEE7C995E1A`, HUGO MAMORI, Guard Coil R1, level1/XP0. Status portrait and identity remain the same after reload into WebGL2; no new starter was issued on the visible reload flow.
- APR guide: detailed native portrait in WebGPU at1280; long guide page in WebGL2 at848 and320. At320 the paragraph scrolls while Back/Next/Close stay reachable. The enlargement did not push the controls outside the panel. Expression/blink animation remains covered by native extraction tests; static screenshots alone do not establish animation timing.
- WebGL2 issued status portrait inspected at1280. No warnings or errors were returned by the browser's captured warning/error log query for this final tab.

The two final normal browser scenes establish this portrait path works on both renderers. They do not constitute forced initialization-failure, context-loss, performance-soak or physical-phone testing.

Final Root validation:538 native workspace tests passed at50b710d;37 portrait-focused checks are a subset, not extra tests.13 exact asset checks passed at8d9c29a.346 shell tests passed with one pre-existing Windows privilege skip at8479398.42 dual-WASM Rust serializer/SaveStore fixture cases passed at50b710d using fake-indexeddb; they are not earned gameplay. Normal release build/TypeScript passed at8d9c29a. Strict native Clippy passed on the integrated renderer before the role-only correction, and the independent reviewer ran it again for the two new native tests.
