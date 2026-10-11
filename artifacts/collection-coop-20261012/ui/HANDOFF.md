# Native collection sports UI — frozen handoff

Source `bad0fb3b3fcd98bd8a6ee999a25def9706307762`, branch `codex/collection-sports-ui`, based on `5aa5cfaeb4db4e71511ac4dc251ce17d1b423923`.
Exactly three owned source files: collection_ui.rs, collection_ui_tests.rs, and the narrow
ui_scene.rs registration/semantic routes. No core rules, catalog/save data, shell code,
shared manifests, portrait/MEISHI/title/icon source edits, preview, browser or WASM build.

## Art and measured interface contract

Native collection entries receive a calm opaque graphite surface, short selected/focus cue,
quiet metal lower separator and locator accents. This is nine occupied cells/nine transforms
per entry, rather than another bright closed frame. The current measured root and content
rectangles remain authoritative. Six original cached 32×32 symbols have independently
authored source motifs: pulse, shield lattice, annular rotor, and separate PAN/RENKO/HUGO
lettered circuit crests. Recesses, alloy/platinum facets and gold fasteners remain neutral.
For each symbol, only its four designated emitter cells change with focus time; no source geometry moves.
Reduced Motion, disabled and selected states freeze this modulation.

Actual symbol costs: category 392 / 404 / 396; corporation 384 / 424 / 432, one transform each.
Every source cell remains submitted at continuous fractional fits, including the tested
2×2 tiny rect; there is no cell-size floor, raster resizing or detail LOD. Recommended host
symbol rect is 16 logical units. Whole-element pixel/work admission and finite all-corner
preflight happen before append. Hidden clipped cells still consume both quotas; reflected
and sheared parents compose exactly. Entry surfaces intersect their clip; symbol cells
require their complete four local corners inside it. Invalid/singular parents and invalid
symbol tokens fail closed without quota/frame mutation.

Existing UiNode fields suffice; no payload/schema additions:

| Kind | value | Other state |
| --- | --- | --- |
| collection-entry | -1 neutral; 0 offense / 1 defense / 2 passive | selected is actual active hull/equipped slot only; active is real focus/hover |
| category-symbol | 0 offense / 1 defense / 2 passive | Never a manufacturer inference |
| corporation-symbol | 0 PAN / 1 RENKO / 2 HUGO | Host verifies actual corporation/point issuer |
| collection-state | 0 found / 1 blueprint / 2 owned / 3 active or equipped | Text remains exact supplied copy |
| collection-name/meta/cost/reason | No gameplay interpretation | Platinum heading/cost, quiet metadata, restrained amber reason |

The renderer has no discovery catalog, prices, level gates or compatibility logic. It paints
only the host's supplied nodes. Manufacturer and construction currency remain separate.
Owned versus effective rank and exact material/component recipes remain literal authoritative
text. Root-owned shell hooks `866778f` and pressed-card correction `8931c69` were inspected
read-only: filtered HUD entries, measured text emitted once, issuer symbols mapped explicitly,
and card selected state restricted to actual data-active. Their shell tests are Root evidence,
not reruns by this Art task. Unknown manufacturer data is not synthesized here.

## Actual verification and cost

- `cargo test --locked -p game_web collection_ui:: -- --nocapture`: nine passed.
- `cargo test --locked -p game_web --lib`: 267 passed, zero failures.
- `cargo clippy --locked -p game_web --all-targets -- -D warnings`: exit 0.
- `git diff --check`: passed before commit; checkout clean after commit.
- Actual native UiScene fixture: three complete supplied category entries, names,
  owned/active rank, state, cost/reason copy and actual semantic action routes, five current
  64-cell controls and largest DORA portrait. Exact joined-versus-isolated packet equality,
  49,147 occupied cells / 329 transforms. Every batch obeys 128 transform slots.
- Shared 60,000-pixel / 120,000-work quotas remain unchanged. Both quota boundaries,
  whole-element rejection, partial clip, fractional/reflected transforms, semantic text geometry,
  positive/invalid token routes, emitter-only animation and stable Reduced Motion are tested.

`packets/collection.json` was exported by the registered native test at this exact source.
`native-software-collection.png` is its 4× supersampled/BOX flat unlit approximation and was
actually viewed. It is a declared fixture, not actual DOM layout, GPU or browser evidence.
The screenshot shows clear name/state/rank/cost/action grouping at the posed viewport;
actual responsive action readability and compact scrolling remain Root's browser gate.
No physical phone, subjective user acceptance, performance or soak result is claimed.

The existing title, MEISHI, portrait and flight-icon sources are byte-pinned unchanged in
`manifest.json`; their inherited native regressions pass in the 267-test library run.
This is a bounded collection styling milestone, not completion of the broader asset overhaul.
