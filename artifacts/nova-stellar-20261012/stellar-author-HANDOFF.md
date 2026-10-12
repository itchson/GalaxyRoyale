# Irregular seeded stellar neighborhoods

Frozen source `abb339b59c3d185cbc8342f2c1eeed1c540cfaa9`, parent `36e08b75cf287bb43adf684008cb725c3d9eb2a0`, on isolated `codex/stellar-neighborhood-art`. Exactly two tracked files: `crates/game_core/src/cosmic_background.rs` and new `stellar_neighborhood_tests.rs`. Tracked checkout is clean; no push, preview, browser, WASM build or Root source mutation was performed.

The source-backed issue was a repeated strip: every neighborhood placed seven stars in ordered 20-unit axial bins. The new field-local positions vary between compact and loose asymmetric arrangements, with independently seeded span/aspect and point coordinates. Seven stars remain seven native quad cells, not an imported raster, chart line, billboard or new glow object. The stale connecting-asterism comment was corrected.

## Bounded implementation contract

- Birth hash, `h % 17` selection, origin jitter, orientation, `.27` parallax and the 768-unit field tile are byte-identical. There is no gameplay RNG, entity, collision, progression or save change.
- `draw_system` is byte-identical: seven submitted cells/one transform, warm/cool palette, alpha/brightness behavior, Reduced Motion freeze and whole-group rejection at six remaining cells.
- Source span varies from 52 to 120 units; aspect from `.64` to `1`. The rotated rectangle's largest corner is at most `sqrt(60²+60²)`; adding a whole star's half-diagonal remains below the existing 90-unit culling envelope. No culling radius/cap is raised.
- The new sampler uses a fixed `[Vec2;7]` stack and at most sixteen candidate attempts per star. Worst case is 112 candidate positions and 336 pair-distance checks per admitted field neighborhood. It prefers an eight-unit gap; on exhaustion it chooses the largest sampled gap rather than entering an unbounded retry. Eight-unit spacing is a preference, not a proven minimum over every possible hash. The sampled native regression also checks that point footprints do not overlap.
- The new layout calculation allocates no heap and creates no new animation cache. Existing FrameBuilder batch/vector allocation remains existing behavior. This is not a performance measurement.
- Custom 8,000/4,000 quads, 1,800/900 star reserve, regional 6,000/3,000 and gas 800/400 caps are unchanged. Nebula, galaxy, remnant, independent stars and region/planet source are untouched.

`source-boundary.json` verifies the whole old/new cosmic source is byte-identical outside the stellar section/comment and new test registration. The native before snapshot's stellar section matches the base Git blob exactly. No accepted cloud/remnant evidence was amended.

## Actual regression and checks

The native geometry regression failed against the real old sampler before applying the correction: **60 neighborhoods, zero broad shapes, diameter 111.80514–133.89694** (`baseline-morphology.log`). It measures rotation-independent covariance and physical diameter, so merely shuffling old strip points cannot pass.

The new sampler yields **39/60 broad shapes, diameter 42.65442–137.45029**. The test requires a meaningful fraction of broad neighborhoods and variable spreads rather than exact hardcoded nodes. It also checks determinism, finite bounded points and distinct visible footprints. The second new test checks actual FrameBuilder packets, exact short-budget rejection preserving prior content, full/reduced/frozen identity, one transform and four-corner parallax under camera movement.

Actual final commands/evidence:

```powershell
cargo test --locked -p game_core stellar_neighborhood -- --nocapture
cargo test --locked -p game_core cosmic_background -- --nocapture
cargo clippy --locked -p game_core --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Focused result **3 passed / 1 explicit writer ignored**. All scoped cosmic result **32 passed / 5 explicit writers ignored**. Strict all-target core Clippy, format and diff checks passed. The exporter separately passed once on the baseline and once on the candidate:

```powershell
$env:STELLAR_PROOF_DIR = '<owned absolute output directory>'
cargo test --locked -p game_core export_actual_seeded_stellar_neighborhood_packets -- --ignored --exact cosmic_background::stellar_neighborhood_tests::export_actual_seeded_stellar_neighborhood_packets --nocapture
```

The initial test helper compile needed a closure for `Vec2::to_array`; this was repaired before the retained baseline run. An initial offline contact pass used an over-precise float filename and stopped before composing the sheet; final `software-final/` uses explicit one-decimal zoom names. These were proof-tool fixes, not weakened runtime assertions. Formatting was corrected to the workspace's edition 2024 before commit; only the authorized source section/registration remains in the diff.

## Actual native packets and software inspection

`before/` and `after/` each contain twelve native exports: six identical seed/grid/hash examples × full/reduced. Their **24 packets / 72 zoom views / 504 submitted cells / 2,016 finite corners** were independently parsed by `render_contact.py`. Before/after hash/origin/camera/color/material are exact; full/reduced positions/corners are exact. Actual fractional cell widths are `.69`, `1.15` and `1.84` at zoom `.6`, `1` and `1.6`; no minimum-pixel clamp was added. Maximum native source-center projection error over these far field examples was **0.00699237 logical units**.

I inspected `native-software-contact.png`: old examples have repeated long axial envelopes; new examples include compact clumps and broader loose arrangements, with no connection strokes or extra decoration. This is deliberately sparse scenery. `software-final/` contains the 72 declared zoom images.

The contact consumes actual native FrameBuilder/CameraView corners and RGBA. It composites hard quad polygons at 4× in Pillow over `(2,3,5)`, then BOX-downsamples. It does **not** run WGSL, GPU blending, bloom, lighting, whole-scene point density or a normal browser. Those images are software evidence, not mock screenshots or actual GPU acceptance. The initial float-named software files remain scratch history in `software/`, separate from the declared final sheet.

## Root reproduction targets

Actual native seed **871238449**. These positions derive directly from the exported native field origin divided by `.27`; they do not change the seed or clock. Normal Root QA WARP can center the selected neighborhood, but the full scene also contains independent stars and foreground objects. Moving the craft/viewport slightly away from the center can avoid foreground occlusion.

| Form | Grid / hash | Native camera / console WARP | New diameter / covariance ratio |
| --- | --- | --- | --- |
| Compact | `[-7,-16]` / `1377129642` | `WARP -18795 -44326.41796875` | 70.84 / .455 |
| Loose | `[-13,-15]` / `576302822` | `WARP -36179.88671875 -41401.5546875` | 137.45 / .494 |

Suggested bounded actual view: confirm existing Profile seed, use normal console WARP, close console, capture settled 1280×720 at zoom1/1.6, and compare those two neighborhoods plus Reduced Motion and reduced density. Keep seven points subordinate to the craft and separate from navigation/UI motifs. This is a proposed capture plan; no such browser action was performed here.

Actual full-scene visibility, repeated-neighborhood perception, moving views, physical-device cost and all-seed contrast remain Pending Root browser/user review. The earlier comet continuity plan/proof and frozen comet form image remain unchanged; no moving-comet pass is inferred from this source work.
