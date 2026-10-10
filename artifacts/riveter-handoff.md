# Iron Belt Riveter — integrated handoff

The ordinary Iron Belt Bastion variant is implemented in `crates/game_core/src/biome_enemies.rs`; root integrated the shared `lib.rs` hooks and browser diagnostic whitelist. `work/riveter-integration.patch` records those hooks, including the final birth-site biome lookup. No manifest, save schema, browser or shared-lib edits were made by this worker.

## Actual behavior

- Family weights still come from the player's current biome. After a successful ordinary spawn, one third of eligible Bastions born in the actual Iron Belt biome receive a permanent-for-this-entity Riveter component and its birth grid. Crossing a biome border never changes that identity. Bosses, champions, arenas and QA scenes do not receive automatic variants.
- The 40×40 authored body has 856 occupied cells, 15 opaque colors, two 100-cell barrel masks and a derived collision radius at the normal Bastion scale of 0.7. Rendering uses its live `PixelBody` cells and normal material response; the PNG is preview-only.
- An initial charge starts only when the full hull and a 28-world-unit warning fit inside the actual unshaken camera viewport. Charge lasts 0.9 seconds; aim stops tracking for the final 0.25 seconds. Amber barrel highlights and short cell rays show the same direction used by the shots. Three rounds fire at approximately 0, 0.12 and 0.24 seconds, followed by 4.14 seconds of recovery. Fixed-tick transitions add at most small tick rounding to the approximately 5.4-second cycle.
- Each round is a normal hostile projectile at speed 125 with damage `actor.damage × 0.8`: 7.2 in the declared tier-one fixture. The normal Bastion's tier-scaled hull/contact damage, pursuit range, standoff motion, spawn count and shared projectile cap remain in force. Generic enemy updates skip the variant to prevent duplicate shots.
- Muzzles come from actual surviving barrel cells: select the occupied front-row cell center nearest the row's mean, choosing the left cell on a symmetric tie. One lost barrel falls back to the other; both lost barrels disable shots without regenerating cells. No invisible pristine muzzle is used.
- Defeat uses the existing `kill_actor` path: one Bastion family kill, five selected-ship XP, at most the normal ordinary-enemy random drop and one accepted Bastion bounty increment. It reserves the existing 24-kill generation-one queue normally; it adds no bonus currency, counters, rewards or spawns. Retreat uses normal removal and does not grant progress.
- Haven arrival clears the enemy through existing dock handling. Remote sanctuaries reset warnings, stop shooting, disengage and push the complete body outside the protected radius; shared swept projectile safety still removes crossing rounds.

## Authored assets and provenance

`assets/pixels/riveter-source.py` defines the original cell geometry and palette, regenerating `riveter-atlas.json`, `riveter-preview.png` and `riveter-provenance.json`. It uses no AI image/model and loads no raster into the game. The nearest-neighbor preview was inspected during this task. Pillow is used only by this optional asset-authoring script; runtime adds no dependency.

## Observed validation

Ten new portable native regressions pass. The full integrated core suite passed 75/75; following the final explicit symmetric-hole tie assertion, all ten variant tests passed again. Native all-targets and WASM strict core Clippy pass.

Tests exercise actual controller projectiles reaching the player's occupied hull, the three-round locked direction and absence of generic extra fire, actual friendly swept-pixel-hit defeat, once-only kill/XP/bounty credit, other-family generation preservation, exact damaged render cell counts, live charge under Reduced Motion, one/both barrel destruction, shared projectile saturation, actual camera-gated first charge, range retreat, Haven cleanup and remote-site/through-site safety at 0.05-second ticks. The `biome-riveter` diagnostic selects a real deterministic Iron Belt location, suppresses normal spawns and incidental rewards using the existing QA scene flag, and its real projectile defeat preserves the profile and produces no loot in the declared fixture.

## Pending acceptance

This is one implemented biome enemy, not a complete six-biome enemy roster or radiation hazard system. Native fixtures deliberately use ordinary tier-one actor stats to isolate behavior; they do not establish natural distant-biome difficulty, human dodge balance, browser/mobile warning readability or physical-device performance. Root owns normal browser integration/build checks. A short human pass should observe entry at both zoom limits, identify the final locked warning, dodge one burst on fresh Scout, and confirm a destroyed barrel's shot direction before any tuning.
