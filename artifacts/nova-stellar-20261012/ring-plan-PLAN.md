# Three ringed-planet orientation capture targets

Read-only source plan for Root `ab71442b9540c3f33c61a418d445323a1b9ccc1e`, seed **871238449**, settled zoom **1.6**. No runtime edits, native tests/exporters, builds, listeners, browser actions or artificial time were performed. Root owns normal GPU/GL capture and any later implementation decision.

All three bodies are ordinary region layer1/type1 gas planets, with birth biome **not4**, so the isotope-scar replacement does not apply. Their hashes are distinct, divisible by3 and even; the Reduced Density stable-half gate retains all three. Full/reduced use the same birth, palette, parent angle and nominal world footprint, but reduced occupancy/shades are coarser; this is not exact cell/color parity.

## Executable poses

The public QA console `WARP` values below place the **camera400 world units right of the birth-center camera**, keeping the planet about88 world units left of the craft. Do not change/import/reset the seed to produce these targets.

| Target | Grid / birth hash | Birth biome | Current scale / rotation | Unobscured console command |
| --- | --- | --- | --- | --- |
| A, established reference | `[-1,1]` / `2928467406` | 0, Haven Expanse | `.64223135` / `-.2` rad | `WARP -843.728515625 5313.4267578125` |
| B, smaller second body | `[-3,-1]` / `2525719458` | 5, Cryo Rift | `.53324330` / `-.2` rad | `WARP -9028.4736328125 -1413.2913818359375` |
| C, similarly sized third body | `[-4,0]` / `2113593648` | 5, Cryo Rift | `.63148338` / `-.2` rad | `WARP -13312.6630859375 1243.4149169921875` |

Actual source-derived relative centers are A `[-88,0]`, B `[-87.9990234,.0001221]`, C `[-88,-.0001221]`; the small residuals are the source's f32 parent/camera cancellation. The true center-camera for A is `[-1243.728515625,5313.4267578125]`, matching the previously retained native QA log exactly. For B/C, subtract400 from the table's X to center the planet; that optional view overlaps the central craft and has not received the offset-pose quota analysis.

Three different selected hashes avoid comparing only mirror-grid instances that happen to share one hash. Current source still gives every ordinary type1 planet the identical cached albedo/storm and `-.2` parent angle. That is source evidence of reuse, **not a finding that the three normal views look objectionably repetitive**.

## Visibility and quota calculation

`targets.json` records exact wrapping-u32 hash operations and explicit f32-rounded origin/scale/biome/camera math from the pinned source. It computes the camera-independent birth biome through the same nearest-region-center rules, not the current HUD label. Here the offset camera also lies in the same named biome, but renderer routing uses the birth biome.

The plan initially evaluates logical surface533×300 at zoom1.6, giving world extent333.125×187.5. A conservative rotated nominal-body/ring radius is `scale*sqrt(64²+24²)`: **43.898 / 36.448 / 43.163 world units** for A/B/C. This intentionally overbounds the actual occupied ellipse and sphere. At offset center−88 the largest complete nominal bound remains inside that viewport; the rings do not depend on a one-pixel minimum or a bigger culling radius. The production planet visibility test remains135 world units.

For each offset pose, the modeled source scan has **no visible earlier layer0 remnant or layer1 planet/scar before this group**, in either density. At533×300 there is only this selected regional group. Additional checks at640×360 and800×450 also find no earlier eligible regional object. Time-dependent gas runs first but can consume at most800/400. Thus at least5200/2600 of the normal6000/3000 region quota remains before the selected group, sufficient for the established **5004 full /1364 reduced cells, three transforms**; leftover margin is at least196/1236. Later groups cannot retroactively remove it.

Those counts come from the retained real native ring packets. Current `regions.rs` is **byte-identical** to the tested ring-shadow source `87be4b8232e7022ceadb91c938eb4d181e092e4c`, so occupancy/admission code is unchanged. This calculation is a source-model admission bound, not a new native/GPU test. It does not prove foreground enemies/rocks cannot occlude the view, successful GPU submission, physical device performance or eventual visibility after a different resize.

**Raster sizing matters.** Current `RasterLayout` uses backing pixels and an integer `round(backingHeight/360)` UI scale. At1280×720 CSS a DPR1 surface produces640×360 logical; DPR1.25 produces533×300. Other DPR/window sizes can differ. Main clamps DPR to3 and native clamps backing size to adapter limits. Before capture, use the existing console backend/size readout or DOM-backed canvas dimensions and `--ui-scale`; do not read app globals. For example, observed `#universe.width/height`1600×900 implies533×300; backing1280×720 implies640×360. These two and the larger800×450 extent have the recorded no-earlier-object bound. If the actual extent differs, keep the WARP target but reassess the scan/quota instead of asserting universal admission.

## Bounded actual capture procedure

1. Confirm the normal loaded assets and actual `PROFILE` seed871238449; use solo QA mode outside active co-op. Keep1280×720 and settle zoom1.6 through existing Settings. Record actual backend/logical or canvas dimensions and density state.
2. Enter target A through the actual native console. Close it and wait for a settled frame, then capture the persisted image with the object left of the craft. Repeat B and optionally C at the same settled size/zoom. These are separate naturally seeded objects; no time-matched moving comparison is implied.
3. Through actual Settings enable Reduced Density and repeat the same pose(s). Even hashes preserve selection identity; expect the declared lower-resolution cache rather than full cell parity. Restoring the previous setting is Root's final cleanup.
4. Compare ring-axis angle, cached storm/band orientation and overall silhouette in the saved images, while considering scale/palette variation. A shared angle can still be acceptable composition; only a concrete repeated visual motif justifies the later art change.

The ring surface/angle is cached and static, so no precise clock window is needed. World actors, foreground occlusion, camera interpolation and screenshot settling still matter. `WARP` is the existing diagnostic path: it rebuilds the transient scene, suppresses rewards and can discard carried material/forfeit a duel; use Root's already assigned QA session. This is not naturally traveled/earned progression. No hidden state, seed override, co-op guest scene or browser app-global mutation is part of the plan.

## Minimal proposed variation, not implemented

If the three actual views confirm unwanted repeated orientation, change only ordinary type1's parent angle to a modest stable sky-axis variation, for example:

```rust
-0.2 + (unit(mix(h ^ 93)) - 0.5) * 0.5
```

This ranges−.45 to+.05 radians, reuses existing hash math and rotates the complete sphere/back-ring/front-ring composite. Source-model values for A/B/C would be−.4049601 /+.0098429 /−.0085866; the first two give a useful contrast. It keeps the baked sphere/shadow relationship aligned, palette/alpha/cells, three transforms, selection/birth, quality identity, parallax and old conservative culling bound. It introduces no time animation, shader, new sprite or gameplay change.

This is **two-dimensional sky orientation**, not a new physical ring inclination or simulated orbital obliquity. The baked light direction rotates with the cached composite, as existing non-ring planet parent rotations already do. The ellipse flattening and storm map remain shared, so angle variation alone does not remove every potential albedo repetition. A future finite cached albedo-variant pass would be a separate, larger reviewed scope.

Any implementation should follow Root's actual multi-object evidence and explicit file approval, then test deterministic differing angles, full/reduced identity, finite whole-cell corners, protected old footprint bounds, back/body/front order and unchanged complete quota admission. No planet implementation or visual acceptance is claimed here. The approved nebula/remnant and completed stellar sampler work remain unchanged.

## Evidence boundary

`derive_targets.py` reads ten exact Git inputs and writes only this owned work folder. It verifies the retained actual native A oracle, derives fourteen eligible paired-density grid poses and selects three distinct hashes. The generated `targets.json` includes all modeled scan candidates/alternative extents. This is a Python source-formula derivation with explicit f32 rounding, not native Game execution. `verify.py` checks frozen inputs/evidence read-only.

The earlier native QA log, old ring source/contact/packets and prior material-repair review remain frozen separately. Moving comet continuity, whole-universe quality, physical phones, performance/soak and user art approval are not inferred from this plan.
