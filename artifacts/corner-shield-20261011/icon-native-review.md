# Independent native 64-cell control review

Reviewed on 2026-10-11 against Root source commit
`6f49be9dfcbbf9cafaf1b154fe14033fb000ffde`. The reviewed source files are
committed at that boundary; the live Studio ledger and unrelated evidence
outputs remain outside this review. The atlas is byte-identical to authored
asset commit `749b85a6a08d42330d3aa0b115ebe7d7bd8e88ea`.

**Result: no remaining source blocker in this bounded integration review.**
The focused actual native checks below pass. This is not a browser, human
readability, physical input or phone-performance acceptance.

## Cells, masks and landmarks

`flight_icons.rs` resolves exactly ten expected keys once through LazyLock.
It validates the 64×64 dimensions, row-major index length/range, transparent
index0, opaque nonzero palette entries, unique occupied mask cells, emitter
subset relation and bounded500–3,000-cell objects. The frozen authoring pack
has21 palette entries and stronger authored mask/count/landmark validation.
Every object is a cached PixelBody; all transparent apertures and disconnected
parts survive. No bitmap resampling, compressed-detail LOD, coarse mask
substitution or minimum-cell-size clamp appears in the native route.

| Icon | Occupied | Accent | Emitter |
| --- | ---: | ---: | ---: |
| Status | 1,291 |113 |18 |
| Map |2,036 |178 |15 |
| Settings |1,802 |100 |36 |
| Save |1,503 |36 |12 |
| Close |1,289 |98 |16 |
| Resume |1,362 |156 |26 |
| Restart |1,373 |97 |18 |
| Title |1,591 |51 |18 |
| Quit |1,322 |100 |27 |
| Fullscreen |1,140 |312 |76 |

The ten total14,709 occupied cells; status/map/settings/save total6,632.
The native landmark test independently names solid/open coordinates from the
authoring designs, including the open settings bore, save aperture, map seams,
restart center, power gap and four fullscreen locator arms. It also extracts
each object at a14-logical-square rect, requiring the complete occupied count,
every original local cell position and exact14/64 transform scale. This proves
preserved submissions, not that all64 cells receive physical raster samples.

`ui_scene.control_icon` uses continuous min(width/64,height/64) fit, composes
the full local affine with its parent, and keeps icon center within the
measured rect. Actual focused tests cover44-physical-pixel rectangles at
UIScale1/1.375/2.4/3/4/6, tiny/far rects and reflected/sheared parent geometry.
Four-corner clipping masks whole source cells to alpha0; it still charges every
occupied source submission to both remaining-cell and work budgets. Each
insufficient budget rejects the entire body without consuming a transform.

## Material and state behavior

Enabled immutable metal/secondary cells keep exact authored colors.
Hover/focus and press affect only accent cells; the explicit
`pressed && marked` branch preserves unmarked colors. The current material
regression now asserts that preservation under actual pressed state as well
as hover/time. Only explicit emitter cells vary over time while hovered or
busy; all other occupied cells remain time-stable. Save pending alone marks
busy, and saved/pending/error accents retain distinct green/cyan/red meaning.

Reduced Motion removes time variation while retaining relevant static
interaction/state color. Idle and disabled objects remain time-stable.
Disabled whole-body muted coloration is deliberate, separate from the
enabled-material promise. Nonfinite time is normalized and extreme finite
time stays bounded; node/clip geometry validation fails closed.

Signals currently animate opaque RGBA brightness through the default unlit
PixelMaterial, not shader bloom or real-time emitted lighting. Native selected
state exercises press styling; physical pointer timing and browser focus
remain Root's actual-browser scope. The bridge currently supplies selected
from CSS :active and semantic persistent states; this source review does not
claim a real pointer press was observed.

## Shared budget: realistic menu versus deliberate overload

The60,000 UI submitted-cell/120,000 work limits and128-transform batch boundary
are unchanged. These are per-UI-draw limits; preexisting unrelated frame cells
are not falsely included in that UI cap proof.

The cap/batch test has three actual starter bodies (662+688+1,040=2,390),
all ten controls14,709, then repeated full DORA objects. Controls total17,099;
one DORA makes51,693 UI cells. Whole portraits are admitted/rejected and batch
indices remain valid across the existing127 transforms plus later objects.

The representative current five-action status fixture uses
close/resume/restart/settings/title, whose icons cost7,417. It also includes
dense panel/text/RAICHO/action content and costs18,642 before the portrait.
Actual printed totals are:

- DORA34,594 +18,642 =53,236;
- DORO28,524 +18,642 =47,166;
- TALO24,262 +18,642 =42,904.

The deliberate all-ten-symbol variant costs25,934 before the portrait.
Adding DORA requests60,528, so the native renderer correctly keeps25,934
prior cells and rejects all three portrait layers atomically. Its prior
pixels, transforms and batch count stay unchanged. DORO/TALO fit that overload
at54,458/50,196. This is a useful negative case, not a reason to increase the
cap, drop accepted face detail or call the all-ten fixture a real modal.
The native fixture is conservative representative content, not a measured DOM
snapshot. Root must separately verify that actual visible creator/dialogue/
status composition does not unintentionally exceed its budget.

## Executed focused checks

All commands ran from Root workspace on the reviewed source. No broad suite,
normal build, browser, server, API mutation or publication ran for this review.

| Command | Actual result | Output |
| --- | --- | --- |
| `cargo test --locked -p game_web ui_scene::icon_tests -- --nocapture` |12 passed,0 failed |[icon log](native-review/icon-tests.log) |
| `cargo test --locked -p game_web flight_icons::tests -- --nocapture` |2 passed,0 failed |[material log](native-review/material-tests.log) |
| `cargo test --locked -p game_web ui_scene::kikai_integration_tests::scene_cap_and_transform_batch_boundary_keep_whole_portrait_objects_and_valid_indices -- --exact --nocapture` |1 passed,0 failed |[cap/batch log](native-review/cap-boundary.log) |
| `cargo test --locked -p game_web ui_scene::kikai_integration_tests::high_detail_profiles_and_real_dense_menu_controls_fit_the_shared_scene_cap -- --exact --nocapture` |1 passed,0 failed |[dense-menu log](native-review/dense-menu.log) |

The filtered boss-layout binary reports zero executed tests for these filters;
it is not additional acceptance. Earlier icon authoring Windows/Linux11-test
proof remains historical asset evidence, not a new execution in this review.
Final browser sampling, hit targets, actual hover/press/busy behavior, moving
scenes, subjective quality and physical phones remain separate gates.

## Exact reviewed input SHA-256

These are raw working-file bytes, verified after the scoped checks; Root owns
any later source or build identity.

| Input | SHA-256 |
| --- | --- |
| `assets/ui/flight-icons/atlas.json` | `e86980bbb99f3c1423900283f78e7bba2aac4b90d02b4a0855291c2e5ff8ee73` |
| `assets/ui/flight-icons/manifest.json` | `7a675cb0f343bdfd676b7f6784e84e78aed8db2dc7255a56fbf8a687b6b4aa33` |
| `assets/ui/flight-icons/provenance.json` | `a0664fcd5e37cb64693fb6646b2cae39b6b4e260c0b178a72ce4e87345a7f7ed` |
| `crates/game_web/src/flight_icons.rs` | `ba02a9aa4e210f5a2746d66af1b7d6bcf0c5ebd89ec3241f00e790f88cbf2503` |
| `crates/game_web/src/ui_scene.rs` | `30feb18eda281b3d58af1b7167ccc6ac21637f2f41651e80e6ce0f1459df644d` |
| `crates/game_web/src/ui_icon_tests.rs` | `c12d1f0bda3cc94f0781df7ecd230edcabb76fdbe5426705f894ee50ae485bd5` |
| `crates/game_web/src/ui_kikai_integration_tests.rs` | `9ba62185dd4be01d016d11f5796512172931739fccf90c3c415ded0fd2815e04` |
| `crates/game_web/src/kikai_art.rs` | `0d0d7fe7e7ae2b453b28a9b48fb301b66f3ee1272e66ab33a0898c3f7e83cbb0` |
| `Cargo.lock` | `df228d9f83ef57321dcb768b79d948e9aa24456c0952e971fc8d6fcd7e806bdd` |
