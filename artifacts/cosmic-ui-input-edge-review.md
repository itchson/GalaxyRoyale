# Narrow touch hitbox refinement

This separate addendum supersedes the two-pixel input-overlap limitation in the earlier immutable `../HANDOFF.md`. Only production `pixel_ui.rs` changed in this refinement. `boss_overlay.rs` already consumes the shared rectangles, so its source remains unchanged. No gameplay, DOM, CSS, renderer, dist or publishing edits were made by this specialist.

The shared joystick hit rectangle is `[14,H−92,(W−98).clamp(0,84),84]`, and the fire rectangle remains `[W−84,H−86,72,72]`. At logical safe widths180/181/182, joystick widths82/83/84 give centers55/55.5/56; both controls retain center heightH−50. The painter now derives both centers from those actual hit rectangles. Joystick/fire hit regions meet without positive overlap at all three widths. Radius28 visuals, proportional stick offset18 and input normalization remain unchanged.

Two durable native regressions check exact shared centers, disjoint hit ownership, every nontransparent emitted cell corner, FIRE glyph occupancy and normalized knob displacement at180/181/182×140/180. They include released, partial, diagonal and invalid stick values. Wider center invariants cover182/200/220/320/533/720. The existing384-scene HUD/control geometry test and inset/boss reservations tests also pass. Genuine180×140 touch boss no-fit remains fail closed; this input repair does not claim a guaranteed100×40 boss panel in an overconstrained viewport.

Actual commands executed successfully:

```powershell
cargo test -p game_web pixel_ui -- --nocapture
cargo test -p game_web boss_overlay -- --nocapture
cargo clippy -p game_web --all-targets -- -D warnings
```

Results:16 pixel_ui tests and9 boss_overlay tests passed; strict native all-target Clippy passed. Root owns complete checks, both WASM paths and browser/device acceptance.

Before Cargo rebuilt the native library, its pre-edit `libgame_web.rlib` was copied to `libgame_web_before_edge.rlib`. The independent `parity.rs` imports production's actual revised `pixel_ui.rs` and calls the retained library's public old `draw_controls` beside it. It compares entire `PixelInstance` and `GpuTransform` arrays, batch counts and pixel counts for540 cases: widths182/200/320/391.75/533/720, heights140/180/300, three cameras including±999k, five stick states and both fire states. All packets matched exactly. This is native extraction parity, not a browser rendering/performance claim.

```powershell
rustc --edition=2024 work/cosmic-ui-hud-review/input-edge/parity.rs -L dependency=target/debug/deps --extern engine_core=target/debug/deps/libengine_core-c6e44304da45d14c.rlib --extern game_core=target/debug/deps/libgame_core-684f3d0a8e530f28.rlib --extern game_web=work/cosmic-ui-hud-review/input-edge/libgame_web_before_edge.rlib -o work/cosmic-ui-hud-review/input-edge/parity.exe
& work/cosmic-ui-hud-review/input-edge/parity.exe
```

Fingerprints:

| File | SHA256 |
|---|---|
| production pixel_ui.rs | f734820d74f925b4b794edb9576217b93a53126d2c6cd3c404842f3cab4ac730 |
| production boss_overlay.rs, unchanged | fd672d8d65ad3c8823fedbe1dd586b53abc9f8e6d81f6949e9a9e7f36ca14b2c |
| copied pre-edit native game_web library | fa4a446463b00735e2f6ede7e8c95888c169d27be3b4e4bad28a3438d962a7e5 |
| actual engine_core-c6e library | 824a4777aa0599625808653ad1a4bb5e9fdba44861046f404dc39e8edefd4a01 |
| actual game_core-684f library | 8b328e4c08d64270dafcd3e0f729be47f6b3803040823782e8e11f07b7b73b47 |
| parity.rs | eeab30800cb96589d5bb5fdfc73cd8f18dc1f955417d4fef15bc5a79efa0162c |
| parity.exe | d3391bcdb9bf27e74195a465156ccfc0da0c2382cd6f23fb9e54845ca772a01d |
| parity.log | a787650adacbce12feca5aecf66fc535cbd0d6d79c0a15f185f1fb8e0dcce3b3 |

Production's CSS owner separately reports the matching joystick width `min(84*uiScale,100vw−safeLeft−safeRight−98*uiScale)` and unchanged height84*uiScale. Its frozen stylesheet hash is bde3cfca8deb788f412bdbce64d83d6c36368d121e41b0779a9f3b767cc2da89. This note records the agreed contract; DOM-to-native geometry and real browser input remain a subsequent root/QA gate.
