# Cursor aim: independent native and bridge review

No blocking source defect found in the frozen cursor-aim implementation. Six root-owned actual Game tick tests and three additional owned actual Game checks pass. The change follows the accepted arbitration contract without altering the save schema, shared projectile path or death/sanctuary ordering. This review changes only files in this work directory.

## Exact reviewed source

`implementation-fingerprints.json` pins 44 source, atlas, manifest and existing native dependency files before and after both compilations. Main identities:

- `crates/game_core/src/lib.rs`: `cc561987f84f908a9dfcd1fbe612fc1de2db5dc6f7b6cb50508a4e6fae81961d`.
- `crates/game_core/src/cursor_aim_tests.rs`: `4abfc3c200c8500a22b334d2d9839386b6f8764bcc0a8c39034d83a1577cdf29`.
- `crates/game_web/src/browser.rs`: `2be9f952d91a3994688076f0c0a57fc468c254bd05073a6f14469156045143c5`.
- `apps/game/src/input.ts`: `f87bcf312e904c9011b73798c407bc8daa77fbc5c3ac34186be60587e5028eb4`.
- `apps/game/src/main.ts`: `1fb17982f1766c81d5671c221c5759c197b6663aa52f60429c8cb233cbf6c7e9`.

The earlier baseline recommendation, immutable pre-aim source excerpt and fingerprints remain separate. No baseline evidence was rewritten.

## Controller and projectile findings

Aim is transient, accepted only while firing, finite and within ±1,000,000 world units, with a direction longer than four world units. Only one turn runs per fixed tick: aim suppresses movement-facing turns, then uses the existing shortest-angle 4π rad/s turn. The independent ±π check verifies the short crossing and preserved previous heading rather than a second turn toward the joystick.

The stored course still derives desired velocity directly from its world delta, with the existing stopping-distance speed and arrival threshold. Aim does not replace that target. Manual stick still clears a course; with valid aim, it derives desired velocity from the stick rather than the nose. Residual throttle follows current velocity while aiming, avoiding acceleration toward the cursor during release. Aim=None preserves mobile heading-fire behavior.

Firing remains after control and before shared integration. The **first shot can leave before the ship has finished turning**. Primary origin is the current nose at +19 world units; primary velocity is nose direction ×430 plus one quarter ship velocity. The owned moving-shot check verifies the actual Bullet previous position, swept new position and inherited velocity. This is deliberate smooth ship turning, not an instant cursor ray or an aligned-shot queue. Body transforms, signature weapons and muzzle lights continue using the same current heading.

The early shared death guard remains before control and firing. Pause/invalid-dt guards, cooldown debt handling, safe-zone firing gates, shared projectile caps and collision integration are unchanged by the aim branch. The six focused checks include sanctuary/pause behavior; this review does not rerun the previously accepted complete death/collision suite.

## Input and browser integration

FlightInput stores held mouse client coordinates and reprojects them each sample through its latest unshaken camera and actual eased world viewport. Right actions do not submit a course; left/right button masks support chords. Mouse state is established before a Haven intent callback can disable/reset input. Touch canvas events do not attach mouse aim to mobile fire. Release, capture loss, cancel, blur, reset and disable clear held aim.

The enabled setter's equality guard makes repeated assignment idempotent. `enabled=canRun` occurs in the recovery transition; it is **not assigned on every RAF**. QA has separately preserved an erratum for that earlier rationale. The final setter itself is coherent.

Main samples input before `Galaxy.step`, then refreshes input camera and world viewport from the completed frame. Thus reprojection uses the latest presented unshaken scene, including its eased zoom, rather than predicting the next follow-camera frame. Browser copies continuous aim across the existing fixed-step accumulator and consumes destination only once. Presentation shake is applied separately for drawing. Modal, death, recovery and pause paths still reset/disable input; Browser pause also clears its destination queue and accumulator.

QA's final 15 EventTarget/input checks and strict TypeScript result were inspected at `work/zoom-tuning-release-review/input-aim-final-tests.log` and its final hash addendum. Those tests exercise the actual FlightInput class with synthetic surfaces/capture methods; they are arithmetic/state proofs, not physical browser pointer-capture acceptance. Its source hash matches this review. QA's final source/package review is separately frozen at `work/cursor-aim-release-review/LOCAL_REVIEW.md`.

## Executed focused proofs

Available Windows Rust 1.97.0 compiled the actual production `crates/game_core/src/lib.rs --test` directly with the five pinned existing rlibs, avoiding shared Cargo targets/manifests. `native-tests.exe cursor_aim_tests:: --nocapture` passed **6/6**, with 125 unrelated checks filtered.

The second harness contains an exact actual-lib copy with only 13 external module paths relocated and one appended owned test module. It imports original source modules and assets. `additional-tests.exe independent_cursor_tests:: --nocapture` passed **3/3**, with 131 unrelated checks filtered:

1. Shortest heading wrap has one aim turn, preserves previous heading, cancels a course on manual stick and accelerates along that stick.
2. A replacement left course during continuous right fire matches ordinary navigation position/velocity every tick for 480 real ticks, brakes and arrives within four world units.
3. A moving first shot starts at the partially turned actual nose, retains quarter velocity and integrates its swept segment correctly.

One initial owned assertion assumed Game::new starts at heading zero; its actual heading points upward. The owned quiet fixture was corrected to explicitly initialize both headings, matching the declared root test fixture. No production defect or source change resulted. Final logs contain the successful exact fixture.

Repeat commands:

```powershell
python work/cursor-aim-engine-review/pin-implementation.py verify
work/cursor-aim-engine-review/native-tests.exe cursor_aim_tests:: --nocapture
work/cursor-aim-engine-review/additional-tests.exe independent_cursor_tests:: --nocapture
python work/cursor-aim-engine-review/freeze.py verify
```

Build commands are retained in `build-native.ps1` and `build-additional.ps1`. The owned source preparation command refuses to overwrite its relocated copy. Test runtime is not an extraction/GPU benchmark, and no tests granted rewards or XP.

## Remaining actual observation scope

Root's `artifacts/cursor-aim-validation.md` reports foreground normal WebGPU and WebGL2 right-held rotation with a real bullet line in declared quiet-space WARP fixtures, and one update-at-Haven path. This reviewer read that report but did not operate those browser sessions; those observations remain root's separately scoped evidence. Root also reports the full 252 Rust/87 shell checks, strict native/two-WASM Clippy and normal build, which are not claimed as independent reruns here.

Broader actual left-course plus sideways right-fire, keyboard strafing, stationary cursor during zoom/camera follow, pointer chords on target browsers, mobile controls and physical-phone acceptance remain distinct from these native proofs. Human tuning, sustained performance and the complete PWA/recovery matrix are not established by this bounded review. No production change is requested.
