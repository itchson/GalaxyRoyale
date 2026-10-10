# Isotope Mote — isolated next-content prototype

This native/WASM portable controller and original pixel concept are **not integrated or published**. Everything lives under `work/radiation-enemy/`; its own manifest, lock and target directory leave production source/manifests unchanged. The ordinary enemy/reward path remains a future integration contract.

## Source-backed intent

Current `regions.rs` calls biome kind 4 **Radial Wastes**, with Dart/Bastion/Coil weights `[7,3,2]`. Radiation rings/gas are cosmetic. Ordinary Darts currently weave and ram; they have no ordinary ranged shot. The Mote is a proposed one-in-four Radial-Wastes Dart birth variant that adds area control. It does not implement ambient radiation, exposure, slowing or a new damage type.

Birth is immutable for the entity: seed, actual spawn-biome/grid and existing spawn-roll bits (`Brain.phase.to_bits()`), hashed once. Family 0 / ordinary / biome 4 are required; boss/champion/arena/QA exclusion also belongs in the host spawn hook. No global serial, saved variant record, new currency or additional spawn is needed. Its shape stays fixed across biome borders.

## Implemented controller

The Mote approaches/orbits at roughly 180 world units. Its first charge waits until the complete hull plus 35-unit warning fits the actual unshaken world viewport. Charge lasts 1.1 seconds and tracks a player-facing 120-degree escape sector; the final 0.35 seconds locks that sector. The triad turns away from the gap so all three pylons own part of the blocked arc. Lime boundary cells mark the escape sector; amber cells indicate pulse lanes.

Two pulses occur 0.35 seconds apart, each with at most six normal hostile shot commands at offsets `80/120/160/200/240/280°` from the locked gap. Centers stay at least 20 degrees outside the marked sector. Speed is 110; damage is supplied ordinary contact damage × 0.6 (5.4 in the declared tier-one fixture). Attack ends after 0.5 seconds and recovery lasts 4.4 seconds, giving an approximately six-second cycle plus fixed-tick rounding and visibility admission.

Shots originate at actual surviving cells. Each nominal pylon owns its sector; destroying it suppresses those rays without replacing it from another pristine pylon. The controller consumes a pulse even when the shared projectile budget is exhausted: there is no deferred burst when slots reopen. Output uses a fixed six-element array. Damage/rendering share the live `PixelBody` and preserve reactor/impact holes.

## Exact integration boundary

`Controller::new(Birth)` returns `None` for ineligible births. `step(&Input, &damaged_body)` returns a guarded pose, bounded shot commands and a retreat flag. Apply the returned position/velocity/heading **before the existing single integrate(dt)**; do not integrate twice. Skip generic enemy AI for an installed component. Keep `Kind::Enemy(0)`, ordinary tier-scaled Dart hull/contact damage and the shared kill/XP/bounty/24-kill queue/drop path. Prototype art scale is 0.6 for its 40×40 cells; derive the host radius from the authored body, as current actors do.

Supply finite live world coordinates, current world viewport, dt ≤0.05, actor damage/scale/tier, remaining **global** bullet slots, Haven plus relevant nearby-site circles, and the host's global `safe_zone_at(player)` as `player_refuge`. That explicit global result handles a protected player outside an enemy's local cache. Use the host's current arena-filtered site guards, post-integration body separation and full swept sanctuary checks, including distant-cache fallback. Do not derive a hazard or safe zone from parallax scenery.

When collecting commands from several enemies, subtract shots already reserved earlier in the same ECS tick from `bullet_slots`. Alternatively, dispatch each enemy's commands immediately and recount the shared cap before the next controller. Existing `spawn_bullet` remains the final hard cap; the prototype does not introduce a separate projectile allowance.

Dispatch commands through existing `spawn_bullet(..., false, 1)` and normal four-second hostile lifetime/global 180-bullet limit (300 in explicit stress). This preserves swept occupied-cell hits, player shields/invulnerability, Repulsor reflection/clear, Pulse Echo damage and Ion Wake damage. Return `retire` through normal ordinary-enemy removal. No health, inventory, loot or persistent progression is changed by the prototype itself; once-only production rewards require later integration tests.

## Authored art and observed evidence

`assets/mote-source.py` authors the shape/palette directly and generates indexed JSON, provenance and a nearest-neighbor PNG preview. The inspected concept has **712 occupied cells, 15 opaque colors and three emitter masks of 142/143/143 cells**. The central reactor opening is transparent occupancy, not painted black. PNG is preview-only; runtime expands JSON into real engine quads. No AI image/model was used. Pillow is only the optional authoring-script dependency.

Actually run: **12/12 native tests**, strict native all-targets Clippy and strict WASM library Clippy. The 2,400-tick/40-second native controller replay observed **5 charges / 60 shot commands / ≤6 commands per tick / ≤732 extracted quads**. Deterministic replay, actual regional identity, final aim lock, both pulses, all three pylon roles, live muzzle destruction, damaged extraction, cap drop/no replay, camera admission, Haven/real remote/global refuge guards, through-circle sweeps and zero/invalid inputs pass. Shared engine geometry tests prove an exposed Scout cell can be hit/damaged and a stationary Scout at the indicated sector center is clear. These are not production `Game::tick` encounter/reward tests or browser acceptance.

Reproduce from repository root:

```powershell
cargo test --manifest-path work/radiation-enemy/Cargo.toml -- --nocapture
cargo clippy --manifest-path work/radiation-enemy/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path work/radiation-enemy/Cargo.toml --target wasm32-unknown-unknown -- -D warnings
python work/radiation-enemy/assets/mote-source.py
```

## Risks and human tuning — Pending

This adds ranged pressure to a family that currently has none. Selection fraction, six-second cycle, 180-unit standoff, pulse density/damage and naturally distant-biome tier scaling need a fresh-Scout playtest among ordinary enemies. The center gap is deliberately safe for a stationary player; circling outside it should be dangerous. Whether that makes engaging area control or a trivial stationary target is unmeasured. Existing shield invulnerability/reflection may reduce successive-pulse danger.

Movement during the lock, multiple overlapping Motes, zoom transitions, tiny safe viewports and destroyed-emitter readability require actual browser and human checks. The host must enforce shared caps/guard coverage without reviving cells or bypassing progression; caller integration and ordinary once-only rewards are not accepted yet. No physical phone FPS, input/audio polish, permanent biome roster or radiation-environment mechanic is claimed.
