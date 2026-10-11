# Shield capacity vitals handoff

Frozen implementation commit: `dfe9312709c7ff831db8ecdf12bafc7212cb3079`, branch `codex/shield-capacity-vitals`, parent `e4fe3f67e665707f190273821273abfc12c040ba` (the separately frozen corner HUD). Only `pixel_ui.rs`, retained `ui_curved_vitals_tests.rs`, and new `ui_shield_capacity_tests.rs` changed. No production Root checkout, core transaction/stat logic, assets, public build, server, or browser was modified.

Each shield mark represents 10 points of actual `Hud.max_shield`; the final mark has the actual remaining capacity. The approved Gameplay contract is normal base30, HUGO with GuardR1 total50, GuardR2 total58, normal maximum132, and Sovereign maximum172. This branch tests those capacity inputs through real native extraction; it does not contain or validate Gameplay's separate new capacity/recharge formulas. Recharge availability is never inferred by the UI.

The hull retains its exact11 marks and original stepped geometry. Shield capacity30 has3 centered marks;58 has5 full marks and one80% capacity mark. At58 maximum/55 current, the final charge is5HP inside an8HP capacity mark; the missing3HP can remain as the existing held damage tail. Capacity backgrounds remain identical during damage. Zero/nonfinite/nonpositive capacity paints no shield track. A fixed18-mark bound represents every current valid profile; malformed finite capacity above180 is limited to18 marks. Extending future legitimate stats above180 requires updating that presentation bound.

Both tracks remain inside the original44Ã—12 logical envelope. All cells use cached local occupied bodies and composed affines, stay unlit, and preserve continuous fractional charge widths. Above11 marks, each complete horizontal field scales together, including its gap. There is no cell-size floor or new per-frame intermediate container/body allocation. FrameBuilder capacity growth and allocation counts were not measured. The shorter shield track follows the existing hull curve at its actual horizontal positions, preserving separation. A capacity change resets presentation ratios/trails so newly added capacity cannot create a false damage tail; actual damage hold, healing, ship/death resets and Reduced Motion behavior remain.

Measured full-charge native submissions (including hull):

| Shield capacity | Marks | Quads | Transforms | Batches |
| --- | --- | --- | --- | --- |
|30|3|196|28|1|
|50|5|224|32|1|
|58|6 (last8HP)|238|46|1|
|120|12|322|46|1|
|132|14 (last2HP)|340|52|1|
|172|18 (last2HP)|396|60|1|
|180 malformed-bound fixture|18|406|58|1|

Actual commands on the frozen source:

- `cargo test -p game_web --locked -- --nocapture`:190 native web tests plus31 existing boss-layout integration tests passed; no ignored tests. Six new capacity tests cover real HUD drawing, truthful capacity/charge slices, original hull cells, compressed positive gaps, reflected/sheared/nonuniform parent corners, zero/nonfinite/above-bound work, animation/reset/reduced behavior, and emitted hull/shield AABB separation.
- `cargo clippy -p game_web --locked --all-targets -- -D warnings`:passed.
- `git diff --check`:passed before commit.

`track-separation-negative.log` is the actual pre-repair one-test failure: at30 capacity an independently shortened shield arc overlaid a hull cell. Final six-test and full-suite logs prove the repaired curve. `focused-tests.log` is an earlier4/5 run whose sole failure was an incorrect test expectation (the fifth charge's right edge is29.5, not30); that expectation was corrected before the frozen six-test suite. It is retained as history, not presented as passing evidence.

`focused-final-tests.log`, `native-web-tests.log`, and `native-clippy.log` are final results. Source snapshots and exact dependency/test linkage are pinned in `manifest.json`; committed source snapshots use Git's LF bytes. This work changes no public bounds export, font, minimap/footer/header layout, boss reservation, shader, packed ABI, or core save.

Scope limits: native CPU extraction only. No current WebGPU/WebGL2 visual acceptance, physical-device test, recharge gameplay execution, performance timing, or natural-play claim. Root owns composition with Gameplay's shield source and normal dual-backend browser verification. Small compressed cells still depend on the finite available raster resolution.
