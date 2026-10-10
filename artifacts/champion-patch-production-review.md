# Champion patch production composition review

Recorded 2026-10-10 20:57 UTC. No blocking composition defect was found in the pinned production UI source. This is a read-only native extraction and source-expression review; it does not establish DOM, GPU, browser, human readability, or physical phone acceptance.

## Identity and scope

The earlier baseline and candidate review remain separate and unchanged in `REVIEW.md` and `review-evidence.json`. The accepted candidate manifest is SHA-256 `cfd6856b8abe0da33950af65c3ff1ec49575dc450b1d292cff3817817cde9f12`. This new review uses root's later production freeze:

| Source | SHA-256 |
|---|---|
| `crates/game_web/src/ui_scene.rs` | `ab20eb16c18d9c30751f679702906ac2aa19b35a9a3f5664ead084224e4401d7` |
| `crates/game_web/src/champion_patches.rs` | `70dca6fb509cc20053bde53b319cf36d3a3e4027f78a7b7354ad1e9ca85907e9` |
| `apps/game/src/pixel-ui.ts` | `4c0e19ec4d317a5586ccb2b3224dd4248328ae4437d85fa9e5b74ba1f87fbc56` |
| Actual native `game_web` library | `f2c0e9c73ec5d62c21c384cb0b95bbadd41cce072136ab0ce6fc864a0794d924` |

The full fingerprints are in `production-fingerprints.json`. An owned copy of the actual production native library prevents later builds from silently changing this evidence. The production atlas and provenance are byte-identical to the accepted candidate. The authoring generator changes only the provenance output filename; the Rust helper changes only its relative atlas include path and formatting. The existing RoyaleMono font source and shared ship atlas are unchanged. This review does not execute the separately integrated game audio or distant scenery runtime.

## Executed evidence

Commands executed from the project root:

```powershell
./work/champion-patch-engine-review/build-production.ps1
./work/champion-patch-engine-review/production-probes.exe --nocapture
python work/champion-patch-engine-review/prepare-production.py
```

The actual native test executable passes **10/10 tests**: four independent production composition tests and six reused frozen author tests. Strict native probe/test Clippy passes. The final pin check confirms the recorded production and candidate inputs still match. Logs are `production-build.log`, `production-proof.log`, and `production-final-pin.log`.

* **416 real production UI node packets** exactly match the accepted helper for all nine selections, invalid/out-of-range values, 24/32 logical rectangles, active/disabled/reduced-motion/time states, partial clips, and near/far camera parents. Pixel and GPU-transform packet equality verifies that the production branch returns before the former whole-body brightening path.
* The installed TypeScript compiler executes the **exact AST-extracted production expressions**, rather than a copied fit algorithm: 108 numeric fractional-scale/both-axis fixtures, eight tolerance-boundary cases, and four non-icon cases. The authored 24/32 logical normalization preserves the measured center, applies only within `1/64` logical unit on each axis, and leaves ordinary text/button/portrait/logo geometry unchanged. The extracted campaign-patch active expression reads the authoritative `dataset.active === "true"` branch without querying hover or focus.
* **216 actual production Rust extraction fixtures** consume those source-derived normalized rectangles with unclipped and partial-clip variants. They verify occupied counts, unit cell spacing, unlit material fields, and half-open center clipping. Raw helper fit still rejects an extent below 24; normalization remains a separate narrowly bounded bridge step.
* A **512-node mixed font/emblem scene** submits 59,989 actual quads across three batches within the single 60,000 scene budget. Glyphs retain unit bases and six-cell advance; all per-pixel transform indices remain valid within the portable 128-entry tables. Fully clipped cells still consume submitted/work budgets, and no private extra patch budget is introduced.

## Source wiring and integration constraints

`ui_scene` forwards the node's rect, affine parent, clip, earned flag, disabled flag, runtime time, and reduced-motion flag to the helper and lends the existing scene's exact remaining pixel/work budgets. Index 8 is reachable through nine-entry selection. The campaign DOM uses champion patch ownership for the first eight emblems and `hud.tournament_won` for the Royale emblem; it does not add a ninth saved champion flag. Ship selection retains its existing asset path.

The UI remains a separate screen-space projection at zoom 1. Group-relative clip centers are evaluated before the parent affine transform; alpha-zero clipping does not reduce the charge. Earned animation changes colors only, and reduced motion freezes that color sweep. The same cached occupied cells remain authoritative.

The narrow fit tolerance is verified numerically, but actual browser bounding boxes and fractional UI scaling must still be observed in the real campaign menu. Center clipping intentionally permits portions of a pixel quad to cross a fractional clip boundary; this proof does not imply a GPU scissor. The current campaign rebuild correctly supplies the earned state. A future implementation that changes only `data-active` in place must explicitly invalidate the bridge, because its attribute observer currently watches hidden/class/disabled rather than that attribute.

Root's broader native tests, WASM checks, release build, and browser captures are separate evidence. This review adds no production edits and makes no phone performance or published acceptance claim.
