# Native campaign ship fitting and semantic text alignment

Frozen on `codex/campaign-ship-continuous-fit` from `0b99d9b2de8da7f14d5d265caf4edee2beb85a48`. Commits `99dd3c0be54043594c92cec633aba65533360c3c` (continuous ship fit), then `fb6e52df9f9241773683eaf539bf55503f30decf` (horizontal text alignment). Each changes only `crates/game_web/src/ui_scene.rs`. Tracked checkout is clean.

Campaign ships now reuse the actual fractional `fitted_body` route instead of flooring the fit or rejecting scales below one. The helper's explicit highlight flag retains disabled-before-active color precedence for campaign ships while licensed hull/component callers retain their prior colors. Source cells remain local; complete-cell clipping and atomic full-body debit of both 60,000 submitted / 120,000 work quotas remain unchanged. No portrait, material, shader, or public renderer ABI change.

Three new ship tests enumerate all ten authored bodies at extents 0.125, 32/2.4, 22.5, 32, 48, and 60 logical units in non-square rectangles; verify source cell retention, continuous transforms/unlit material, all four visible corners, normal/active/disabled source colors, short-pixel and short-work rejection without prior-packet mutation, and full alpha-zero work charges. The baseline actual runtime regression failed with zero submitted cells versus 270 at 0.125 extent. Its first compile had a test-only borrow error; the initial partial-clip fixture accidentally contained an entire narrow ship. Those diagnostic logs are retained and are not passing evidence.

`UiNode.text_align` is additive and accepts left / center / right; omitted, unknown, null, numeric, array, and object tokens fall back left without rejecting the scene. Ordinary `text` plus offense/defense/passive text position each wrapped line horizontally while retaining top alignment. Existing `text(...)` callers retain their former horizontal and vertical semantics, including centered control labels; native micro footer keeps its explicit right-aligned path. Four new tests prove three distinct wrapped-line widths, fractional glyph coordinates, whole-cell scroll clipping and quota charges, tolerant token parsing, and byte-identical centered button packets.

Actual checks:

- Five campaign-focused native checks pass on the first commit.
- Four new alignment checks pass on the second commit.
- All 111 `ui_scene::` native checks pass on the final source, including prior licensed/component/portrait/card tests.
- Strict native `cargo clippy -p game_web --lib --tests --locked -- -D warnings` and `git diff --check` pass.

Evidence remains under `work/campaign-ship-continuous-fit/` and `work/native-text-align/`. These are native extraction and serialization proofs, not browser/GPU/phone readability or current incoming rich-MEISHI art acceptance. No service, normal build, WASM build, Root source, caps, assets, or persistent gameplay data changed. Root owns final bridge integration and both renderer/browser acceptance.
