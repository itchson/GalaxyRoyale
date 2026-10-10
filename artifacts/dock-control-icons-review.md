# Dock/control UI handoff

Production changes are limited to `browser.rs`, `ui_scene.rs` and the new focused `ui_icon_tests.rs`. No input, core gameplay, shell, CSS, DOM, manifest, distribution or publication change was made here. Parent integration and actual browser visuals remain separate.

## Exact contracts

- `Galaxy.dock_facility_at(x: f32, y: f32) -> i32` delegates the actual `Game::dock_facility_at([x,y]) -> Option<u8>` without advancing play. None is `-1`; IDs 0 upgrades, 1 hangar, 2 journal, 3 campaign. Game owns finite-coordinate validation, radius and Haven availability; the bridge adds no alternate hit geometry.
- `close-icon`: cached original 9×9 X occupied object, centered and uniformly fitted inside the supplied node rect. Node text is ignored.
- `save-icon`: value 0 saved/green/check, 1 saving/cyan/dots, 2 error/muted red/X, using original cached 9×9 disk objects. Nonfinite value defaults to saved; finite values floor/clamp into 0..2. Colors are respectively RGB(106,237,193), (88,222,239), (220,125,130). Disabled symbols stay readable opaque RGB(139,155,166); active symbols brighten without moving.
- `facility-label`: existing cached RoyaleMono glyphs, centered font1 in a 100×10 rectangle at `[hitbox.centerX−50, hitbox.bottom+2]`. There is no icon/panel/frame. Default RGB(98,143,156), active bright cyan; root labels FORGE/HANGAR/JOURNAL/ARENAS. Inherited clip must include this label area, rather than only the hitbox.
- `restart-button`: existing semantic action layout and bounded label clipping, with amber RGB(255,215,131). Root should include this action kind wherever actual visible semantic buttons need boss/HUD exclusions.

Icons use local occupied cells and one composed affine transform per object. Integer fitting is used at native/larger extents, and reciprocal integer fitting below 9 logical units. Clipping suppresses a cell unless its entire square fits; all submitted cells, including suppressed alpha-zero cells, still charge both shared pixel/work budgets. A group is omitted atomically when its complete body cannot fit the remaining budget. Original X submits 31 cells; the three disks submit 56/54/56 cells. The existing 60k UI and 512-node caps are unchanged. The symbols are static under Reduced Motion and ordinary time, and their materials remain unlit. No font, bounded wrapper or existing UI kind was replaced.

## Executed checks

Seven new actual-engine tests pass: original object/state distinction, measured/fractional/far-camera centering and all visible corners, whole-cell clipping and atomic budget rejection, disabled/static/nonfinite handling, reflected/sheared parent composition, centered below-hitbox font-only facility labels, and bounded amber restart behavior. Retained `ui_scene` checks passed together with the first six new tests (35 total); the final focused seven and native all-target Clippy then passed. Both final WebGPU and WebGL strict WASM Clippy checks passed after adding the restart kind and query bridge.

Commands: `cargo test --locked -p game_web ui_scene -- --nocapture`; final focused `cargo test --locked -p game_web ui_scene::icon_tests -- --nocapture`; `cargo clippy --locked -p game_web --all-targets -- -D warnings`; both target checks use `--target wasm32-unknown-unknown --no-default-features --features webgpu` or `webgl`, with `-- -D warnings`.

This proves actual CPU extraction and binding compilation, not DOM measurement, GPU appearance, pointer routing or physical-device acceptance. Root owns the normal build and browser facility/save/close/restart interaction checks. The independent wgpu restoration source review is still in progress and does not change these controls.
