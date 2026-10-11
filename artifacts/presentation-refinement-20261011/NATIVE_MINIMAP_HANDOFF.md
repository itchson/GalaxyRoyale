# Native minimap handoff

Frozen implementation: `8408b123343c0d3b6359d9bf77c6abc3cfb289a4`, branch `codex/native-minimap`, base `6152611ec88653946f809f200a24a0de482cf265`. Root has merged this change; this handoff records specialist source/native evidence, not browser acceptance.

The navigation display is now a north-up 44×44 logical pixel minimap on the left, four logical pixels above the footer or touch joystick hit rectangle, whichever requires the higher position. It uses actual `Hud.position`, authoritative home and tracked mission positions, and actual player heading. World +X points right and +Y points down. A fixed 2,400-world-unit range maps to a 17-logical-unit marker radius; distant targets clamp circularly while retaining their bearing. Home is gold `[255,233,133,255]`; the tracked mission is violet `[205,138,255,255]`. A hollow mission marker remains distinct when a delivery target coincides with home. No undiscovered target is added.

The field, stepped pins and three cached occupied marker bodies use real engine quad groups with unlit materials. Only the player child rotates. World zoom changes neither map size nor world range, and UI presentation shake does not change bearings. Shared `FlightStatusLayout.navigation` controls rendering, attached-bar avoidance and boss reservations. Overconstrained layouts omit this optional map while preserving essential footer/vitals. Compact boss mode also omits it when needed to preserve boss health and phase. Normal desktop placement remains available when clear.

New WASM API: `Galaxy.flight_navigation_bounds()` returns a Float32Array-compatible four-value rectangle `[safeLocalX + leftInset, safeLocalY + topInset, 44, 44]` in full logical screen coordinates, or an empty array when omitted. It matches the existing status/text bounds inset contract. Existing strip and text exports are unchanged. The shell must multiply by its actual UI scale for physical facility-label blockers. New bindings must be built before calling this export.

Exactly five committed source files changed: `pixel_ui.rs`, the bounds export only in `browser.rs`, navigation reservations/tests in `boss_overlay.rs`, adapted assertions in `ui_flight_status_tests.rs`, and new `ui_minimap_tests.rs`. No core navigation/save semantics, assets, renderer ABI, audio, shell, build output or Studio records changed here.

Actual final checks (raw logs in this directory):

- `cargo test --locked -p game_web --lib -- --nocapture`: 125 passed, zero failed. Seven new minimap tests and two adapted old flight-status tests are included.
- `cargo clippy --locked -p game_web --all-targets -- -D warnings`: passed.
- `cargo clippy --locked -p game_web --target wasm32-unknown-unknown --no-default-features --features webgpu -- -D warnings`: passed.
- The same command with `--features webgl`: passed.
- `git diff --check`: passed before commit; tracked worktree clean at handoff.

The first full run exposed a compact boss-anchor conflict; the final source omits the optional map in that existing compact mode and includes a regression. Tests cover cardinals, near/far circular clamping, relative positions near ±999,000, actual quad corners/materials/colors, coincident delivery, undiscovered omission, heading-only child rotation, zoom 0.6/1/1.6, insets, header rectangles, full touch hit rectangles, protected targets, and truthful no-fit behavior. Fixtures include 533×300, 320×180, inset-adjusted 320×180, and 180×140. A 180×140 touch screen can omit the minimap; this is not a physical-device support claim. Actual extraction for the tested small map is below 100 submitted quads.

Scoped independent Root bridge review: immutable `a41196dff297426143d1415a4064caaf3faf8068` and `bc5703673bd93ff5f43505ceb0853d44dd336698`. Facility labels now require actual pointer hover or keyboard focus; controls retain their measured hit rectangles/aria semantics. Independently measured settings input/name rectangles pass through the existing scale/clip pipeline. Facility placement accepts only finite, positive four-value minimap bounds and converts them with the actual UI scale; malformed bounds do not become blockers. No scoped source blocker found. Actual compiled/AST checks: `node --experimental-transform-types --test scripts/test/pixel-ui.test.mjs scripts/test/facility-layout.test.mjs scripts/test/flight-status-pixel.test.mjs`, 21 passed, zero failed; `root-semantic-tests.log`. Their working bytes remained unchanged at Root HEAD `45b152c43adba19fcd58655d989c17cf08a30271` when handoff pins were captured.

`frozen-hashes.json` pins raw working-file snapshots, committed Git blobs (including CRLF/LF distinctions), all retained logs and this handoff. `python work/native-minimap/verify.py` verifies owned snapshots/evidence without touching any source. Add `--live` to check currently linked files as well; later legitimate integration changes can invalidate that optional check while the frozen record remains intact.

Pending: Root normal dual-backend build and actual browser/responsive inspection; human minimap readability; physical phones. Native tests/Clippy establish scoped geometry and packing correctness, not GPU timing, frame-rate or phone acceptance.
