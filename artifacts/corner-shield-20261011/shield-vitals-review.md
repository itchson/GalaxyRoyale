# Independent shield vitals review

No scoped geometry or capacity blocker found in Engine commit `dfe9312709c7ff831db8ecdf12bafc7212cb3079` (parent `e4fe3f67e665707f190273821273abfc12c040ba`). Read-only review compared its three-file diff and final source with Gameplay commit `362cd250677e39eb7dcd0ee678e26918615b111e`. No source edit, cherry-pick, package rebuild or browser operation was performed for this review.

## Capacity and charge

`pixel_ui.rs:1140` divides actual `Hud.max_shield` by 10, retains fractional units, and only then takes a ceiling for mark count. This agrees with the real capacity function: normal base30; hardware+10/effective rank; Guard20 at R1/+8 later; explicit Sovereign+40. It does not infer recharge from capacity or grant power.

| Actual maximum HP | Marks | Final mark HP | Independently rerun full-charge quads incl. hull | Transforms |
| --- | --- | --- | --- | --- |
|30|3|10|196|28|
|50|5|10|224|32|
|58|6|8|238|46|
|132|14|2|340|52|
|172|18|2|396|60|

`pixel_ui.rs:1187` clips the background to the final mark's capacity and clips current charge/tail separately. Thus at max58/current55 the sixth mark contains8 capacity HP but5 charged HP. The test at `ui_shield_capacity_tests.rs:96` observes actual packet corners: 60 versus58 loses0.6 logical width; 58 versus55 loses0.9 width, and backgrounds remain identical during damage. At max58/current50 only five marks are charged. These are continuous widths, not rounded HP.

## Geometry and animation

The original eleven hull marks and44x12 envelope are preserved. Above eleven shield marks, the whole horizontal field including its gaps scales together; the18-mark scale is11/18. The shorter shield arc follows hull height at its physical horizontal coordinate (`pixel_ui.rs:1174`), rather than constructing a short independent arc. Actual emitted AABB tests at `ui_shield_capacity_tests.rs:263` cover30/50/58/132/172 plus tiny/zero/110/120/180 inputs and find no hull/shield intersections. Reflected, rotated and nonuniform/sheared parents preserve corners inside the envelope, positive gaps and one batch.

`VitalsAnimation::advance` now resets both normalized fills/trails whenever sanitized maximum capacity changes (`pixel_ui.rs:307`). A30-to50 change with30 current shield gives fill/trail0.6 without an invented20HP damage tail. A subsequent real10HP loss gives fill0.4/trail0.6; held damage settles, healing eases, Reduced Motion snaps to actual values, and ship/death resets remain. This is presentation state only.

Zero, negative and nonfinite maxima paint no shield track. The18-mark clamp truthfully covers every current valid capacity (maximum172); malformed values above180 deliberately cap presentation work, so a future legitimate capacity above180 needs a UI contract update. Current worst full charge is396 quads/60 transforms/one batch at172; the180 malformed-bound fixture is406/58/one. Fractional fills can add split-cell transforms, tested with bounded counts. All recorded materials remain hard/unlit.

## Independently executed evidence and scope

Against the clean, exact frozen Engine checkout, `cargo test -p game_web shield_capacity_tests --locked -- --nocapture` passed6/6 tests with184 native tests and31 boss-layout integration tests filtered out. It printed the table counts above. Runtime310-core proofs are Gameplay's separate execution; Engine's190-web/31-layout/Clippy logs were read as peer evidence and were not rerun as whole suites here. The source manifest pins exact Git bytes and inspected peer logs.

No browser/GPU, physical-display sample, phone, natural fight, recharge balance or human readability judgement is claimed. At14/18 marks the horizontal cells and gaps are fractional; finite raster visibility is Root's actual browser gate. No UI allocation/timing claim is made.

A separate semantic affinity finding in Gameplay362cd25 uses pilot main corporation on crafted hulls. Root approved a distinct repair to use active hull manufacturer. It does not alter the reviewed capacity formula, so it is not a blocker in this Engine geometry commit and is recorded separately from this frozen review.
