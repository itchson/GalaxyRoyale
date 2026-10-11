# Minimal flight presentation handoff

Owned production sources are frozen: pixel_ui.rs, browser.rs, ui_flight_status_tests.rs, new game_core/navigation.rs and four explicitly coordinated additive lib.rs lines. Root owns shell/bridge/CSS; Production owns the ui_scene call to the shared micro helper. No public build, browser, manifest, audio, game art, physics, progression or Studio edits were made by this specialist.

The footer paints text and compact navigation only: no background or border. The invisible 22-logical strip envelope remains the single authority for boss/facility exclusions and accessible contact layout. Both native stats and context use original cached RoyaleMicro 3×5 cells / advance4 / font1. Context export height is now5; its y, strip geometry, hit geometry and paired Float32Array API shape remain stable. micro_text provides full-cell clipping, finite guards and exact caller submitted/work budgets; it allocates no per-frame strings/cell lists after glyph cache creation.

Normal footer navigation reserves100 logical units and shows home plus active quest; narrow navigation uses32 units and prioritizes an active quest, leaving essential H/S/LV/cargo fallback space. Arrow bodies are cached original5×5 cells at eight stable headings; distance is measured from actual world positions. H means Haven; QH means courier delivery to Haven; QC means return to claim at its discovered source; QB means an unfinished bounty's known source beacon, not a claimed enemy spawn location. Completed/relic/undiscovered sites are excluded. Ready claims precede delivery then nearest bounty; canonical ID breaks equal-distance ties. The additive Hud.navigation exposes home and optional quest {position,kind,site_id,name}; it is presentation-only, with no save-schema change or profile writes.

Attached hull/shield tracks keep the shared44×9 logical envelope and actual unshaken interpolated player projection. Nine stepped bevel segments retain continuous fractional fill rather than health buckets. Damage fill changes immediately; the lost cells hold briefly(.22s), then fade by shrinking the trailing fill. Healing eases. Ship changes, dead/respawn states and explicit successful-frame death events reset presentation. Reduced Motion uses actual static ratios. The same core/ship/control/footer exclusions and offscreen culling apply; no forced edge marker or unsafe overlap was introduced. Current low-hull/shield warning thresholds/colors remain intact.

Actual verification retained in this directory:

- 13 focused HUD tests: actual nontransparent corners at533×300,320×180, inset350×200 and tiny180×140; zoom.6/1/1.6; full-cell micro clip under nonuniform rotated parent; cached glyph reuse; no panel geometry; damage/healing/reset/Reduced Motion; fractional segments; compass direction/shake invariants; actual contact hits and essential fallback.
- Full native game_web target:118 unit tests plus31 external layout tests pass. This includes current ui_scene/boss adapter integration, not a browser or pixel-readability claim.
- Four actual Game/navigation checks pass: fresh/no-spoiler, courier delivery/claim, priority/save-order independence and live-position nearest bounty. Supplied profile/position fixtures are explicitly not natural travel or earned reward evidence.
- Strict all-target native game_core/game_web Clippy and both locked WASM backend Clippy pass. No full normal release build was run.
- Current HUD geometry fixtures submit198–481 quads,21–100 transforms and one batch. These are native packet counts for listed scenes, not GPU/frame-time/phone benchmarks.

The existing compact-boss fallback can suppress attached tracks and use the footer's H/S text. Tiny/safe geometry remains fail-closed where no safe placement exists. Narrow navigation can show only one destination; the full discovered map remains separate. Human font/arrow readability, responsive actual browser layouts and physical phones remain Root/Production acceptance gates. The last published cc8d source/archive reviews remain immutable historical evidence.

Read-only owned freeze check: `python work/minimal-flight-native-handoff/verify.py --live`.
