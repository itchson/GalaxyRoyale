# Biome gameplay: current state and one next increment

Read-only source review, 11 October 2026. No production changes or broad test runs. This is a proposal for the Producer, not an implemented or accepted feature.

## What the game actually does

- `regions.rs:12`, `:71` and `:113`: a seeded 4,200-unit jittered nearest-center network supplies six names, background palettes and relative Dart/Bastion/Coil weights. Inside radius 1,400, Haven Expanse is forced. Fixed seed and position reproduce the same biome; no stored biome map is needed.
- `lib.rs:1203–1245`: spawning samples the **player's current biome**, chooses one of the existing three families, and uses the same wave timer/entity cap. It does not sample a new enemy type from a biome. Radial danger and expedition time control difficulty/pressure; biome identity is not an additional HP/damage multiplier.
- `lib.rs:1267`, `:1341` and `:3482`: ordinary enemies have family AI and one base body per family. Darts weave and pursue, Bastions strafe/standoff and shoot, Coils pursue with destructible segments. There is no biome variant component, unique biome attack or biome-specific ordinary hull. Enemy colors stay on their authored family palette; biome palettes color scenery. Asteroid mineral color variants are seeded independently, not unique biome enemies.
- `lib.rs` asteroid streaming: the radial belt at 700–1,400 controls increased asteroid density. The remote biome named **Iron Belt** does not introduce another asteroid belt population or special mineral economy.
- `regions.rs:486–492`: stars, supernova-like features, planets and gas are pure bounded parallax scenery. The source explicitly calls radiation cosmetic. There is no radiation exposure meter, zone damage, protection/resistance, heat status or environmental collision. Gas particles do not slow ships. Moving the cosmetic artwork into gameplay collision would invent new rules and mix parallax coordinates with simulation positions.
- `lib.rs:731`, `:1578` and `:2037`: sanctuary checks, swept occupied-cell projectiles, hull/contact physics and the central kill/reward path are already shared foundations. Haven and discovery sites protect gameplay, with an explicit active-arena exception for its own site. Ordinary bosses and champions have separate controllers.

| Biome | Dart / Bastion / Coil weights |
| --- | --- |
| Haven Expanse | 4 / 3 / 3 |
| Amethyst Drift | 2 / 3 / 6 |
| Aurora Reach | 5 / 2 / 4 |
| Iron Belt | 2 / 7 / 2 |
| Radial Wastes | 7 / 3 / 2 |
| Cryo Rift | 3 / 3 / 6 |

README and ROADMAP accurately leave additional biome enemies/radiation hazards as future content. Current biome pressure and environmental artwork should not be described as those mechanics being complete.

## Recommended next change: Iron Belt Riveter

Add **one actual biome-specific ordinary enemy**, the Riveter, within the existing Bastion family. Its gameplay identity is a readable locked-aim three-shot burst, distinct from the current single-shot strafing Bastion. This supplies a concrete regional combat difference before introducing a separate radiation damage curve into the still-unmeasured Scout economy/survival balance.

Proposed starting contract, subject to human tuning:

- Only successful **ordinary Bastion** spawn opportunities selected while the player is in Iron Belt may produce a Riveter. Select one-third of those opportunities with the existing gameplay RNG, not an extra spawn loop. Keep the regional family weights, spawn cadence and global cap unchanged. The exact same seed, input/tick history and RNG sequence must reproduce selection. Never draw randomness from extraction or cosmetic density.
- Stamp the variant at birth. Crossing into a different biome must not change its hull or attack midway through a fight. Keep `Actor.kind = Kind::Enemy(1)` and `Brain.family = 1`, `boss = false`, with a transient variant/phase component. No Profile/schema/counter array extension is needed.
- Retain ordinary Bastion tier-scaled hull, contact damage, shot damage and speed limits initially. Use its existing approach/standoff physics between attacks. Its attack cycle is **0.90s charge, 0.36s burst, 4.14s recovery**: three shots at burst offsets 0/0.12/0.24s, total cycle 5.40s. Each shot uses the ordinary Bastion's 125-unit/s speed and 0.8× contact damage. This preserves a nominal three shots per 5.4s, equal to the current one per 1.8s, while grouping threat into a dodgeable burst. Matching average shot count does not prove equal difficulty.
- During charge, reduce lateral thrust and hold a visible amber/ivory muzzle buildup. Lock the aim for the final 0.25s; the burst follows that direction rather than tracking each shot. Do not fire before the first charge. Prefer holding the initial attack clock until the actor is inside the camera's playable extent, so this new variant does not repeat the existing offscreen first-charge concern. Ordinary movement can continue before it is visible.
- Author a real palette-indexed 40×40 hull with a recognizable split muzzle and broad armored shoulders, near the base Bastion's world footprint. Derive its conservative radius from actual occupied cells. Maintain hard-alpha PixelBody rendering, exact damage holes, material response and phase-linked charge cues; Reduced Motion freezes decorative shimmer, not charge information. A tint-only replacement is insufficient to accept the new enemy.

## Integration and preserved behavior

Use `update_spawns` for selection, `spawn_enemy` for common actor/body/Brain creation, and a small variant update branch/controller adjacent to `update_enemies`. Both must use the same sanctuary/pursuit gates. The generic controller must skip the variant while its own controller updates it once. Variant projectiles go through `spawn_bullet`, preserving the 180-bullet budget, occupancy sweeps, boundary cancellation and lighting. Its body still uses `damage_actor`; no alternate circle-only projectile hit path.

Every actual Riveter defeat goes through `kill_actor` exactly once as Bastion family index 1: five selected-ship XP, the existing 20% category-drop chance, one Bastion base kill/24-kill credit, accepted Bastion bounty progress and ordinary run accounting. It has no bonus signature chance, extra boss reservation, guaranteed point drop or new material currency. Damage/retreat/reload remain transient like ordinary enemies. Regular boss generation selection, rewards and all fifteen rigs remain unchanged; champion/tournament spawns are excluded.

Reuse `safe_zone_at`, actor push-out and swept hostile-projectile sanctuary cancellation. Do not add a “variant safety” radius that differs from visual/gameplay site radii. Returning to Haven or a discovery sanctuary cancels/defers attacks consistently. The active arena exception still applies only to its owned encounter, not nearby ordinary variants. No radiation/environment side effect is included in this increment.

## Acceptance for implementation and review

1. **Regional identity:** a fixed seed/input fixture produces Riveters only from ordinary Iron Belt Bastion opportunities; other biomes and ordinary bosses/champions keep their existing identities. Variant remains stable across a biome boundary. A natural browser visit observes the distinct hull and mechanic, rather than only a forced QA spawn.
2. **Actual attack:** captured fixed-step traces prove charge → three locked shots → recovery, the correct offsets/cycle and no extra generic shots. A player can observe the initial charge before the first burst; screenshot/video review checks dark-background and touch-control occlusion. No claim of balanced damage follows from timing tests alone.
3. **Progression invariants:** kill a Riveter through real projectile/cell damage; exactly one Bastion credit/bounty kill and five XP are applied. A 23-to-24 transition reserves only G1, surplus persists behind a reservation, and higher-tier queues remain untouched. Duplicate removal cannot pay twice. Diagnostic preview uses explicit no-reward isolation.
4. **Safety/physics:** at Haven and discovery boundaries, no hostile sweep reaches the player and no contact damage bypasses safety. Death/retreat cannot grant a variant kill. Actual hole destruction and circle contact remain aligned with the authored body's transforms. Include long public tick and off-cache sanctuary cases already relevant to the shared controller.
5. **Bounds/migration:** normal entity/bullet/particle caps remain intact; no variant state enters saves, legacy schema-3 snapshots still normalize identically, and rendering at both zoom limits stays finite. Additional art/cue counts must be measured in the existing stress workload before claiming mobile performance.

## Risks and pending evidence

- Burst timing is a proposed initial value, not a tested difficulty improvement. Iron Belt already heavily favors Bastions (7/11 weight); one-third replacement makes approximately 7/33 of all its family selections candidates, without increasing total spawns. Synchronized charges can still be harder than evenly spaced shots; preserve independent birth phases and measure actual overlap.
- The existing Scout study does not test this variant, natural concurrent waves or player reaction. Movement and firing share heading, and category points can lag XP; a variant must not quietly demand an early shield purchase that natural play cannot afford.
- Using the player's biome for selection matches today's family policy, but an enemy's spawn point can fall over a boundary. A future world-population design may prefer spawn-point identity; changing that is a separate decision, not needed for this increment.
- Emitting from destroyed muzzle cells needs an explicit rule. For the first version, use a common surviving-body weapon origin and require at least one occupied muzzle cell before firing; losing both muzzles stops shooting but does not award a kill until hull/body death. Implement that rule only with real cell masks and tests, not a visual-only promise.
- A later radiation milestone still needs seeded **world-space** hazard footprints, onset/escape cues, safety exclusion, resistance/stacking rules and death/XP interactions. It cannot derive damage from moving parallax supernova art. This review does not claim those rules are approved or implemented.

Recommended handoff: one regional variant, one authored occupied hull, one telegraphed controller, existing progression/safety, and explicit human play review. No production source or acceptance status changed during this investigation.
