# Ordinary boss generations

The flight health/phase panel now adapts around actual projected reactors, the player and fixed controls. Compact rows retain the custom pixel font and health meter. See [adaptive boss interface](BOSS_HUD.md) for small-view behavior, tested bounds and acceptance limits.

The three enemy families each have five ordinary bosses. Every encounter has a charge phase with reactor and HUD cues, its own movement/weapon phase, and a recovery window. Charge lasts approximately 1–1.5 seconds and locks the predicted aim during its final quarter. Attack lengths vary between 0.75 and 3.2 seconds; recovery lasts 1.38–1.9 seconds. Low hull shortens firing intervals by 15% while keeping the charge window intact. Natural spawns can begin charging before their bodies enter the viewport; first-charge readability remains a playtest requirement.

| Generation | Dart | Bastion | Coil |
| --- | --- | --- | --- |
| 1 | Matriarch: weaving radial bursts | Prime: aimed five-shot fan | Sovereign: serpentine pursuit and triple fan |
| 2 | Harrier: locked dash and needle volleys | Sentinel: paired side broadsides | Sidewinder: fast sinusoidal rush and fork shots |
| 3 | Reaver: alternating scissor passes and split forks | Gatekeeper: alternating pincer lanes with different projectile speeds | Ouroboros: loops around a locked position and fires inward chords |
| 4 | Tempest: fast orbit with rotating gaps in its spiral | Citadel: anchors and issues radial walls with one open quadrant | Leviathan: broad S movement and volleys from surviving tail segments |
| 5 | Imperatrix: forward/reverse royal cross with rear pressure | Oblivion: slow retreat and rotating fast pulse rails | Worldeater: alternating vortex/lunge cycles, rotating rings and tail volleys |

The bosses use distinct authored 64–96-cell head silhouettes and seven nested rigid visual roles, expanded from `assets/pixels/boss-atlas.json`. Coil chains use 11/13/15/17/19 separately destructible segments, with five authored segment templates and their own visual rigs. Each segment's physical radius, scale and spacing are derived from its actual cells.

New Coil chains extend outward from their head, away from the player, with every initial head/segment facing inward. This applies to base Coils, ordinary bosses and diagnostics. The previous fixed world-down layout could place higher-generation tails across the player immediately after an above-player spawn; regression coverage reproduces that real contact and verifies safe first-tick initialization at sixteen angles and two small viewports. Boss diagnostics still place the head 210 units above the player, which can initially put it outside a short viewport at zoom one.

The render rigs partition the current damaged body; they never restore removed pixels. Static parent-relative placements compose into the same occupied world positions used by projectile hit tests. Reactor/emitter colors and materials animate by role. Independently moving armor hitboxes are not implemented. Contact collisions use conservative object circles; projectile collisions sweep through the surviving pixel cells.

Closed armor takes 55% hull damage during charge and attack. Recovery restores ordinary hull damage, and shots that hit the authored local reactor region deal 165%. Destroying Coil armor transfers damage to its head, with closed-phase mitigation intact. A fully erased body also dies. The HUD reports generation, phase, health and current world-space body/core positions. Champion and tournament encounters use generation zero and retain their separate mechanics.

## Queue and save rules

Every 24 defeated base enemies reserve one generation-one boss of that family. Further base kills accumulate as surplus. Two defeats of a generation queue one boss of the next generation, through generation five. Only one ordinary boss is active globally. The highest queued generation gets priority, followed by family order; its credit is consumed only when it is actually defeated.

Returning to a sanctuary, outrunning pursuit, dying or reloading preserves an unfinished queued fight. Higher bosses never consume a reserved first-generation fight. For one family, 384 base kills can supply the complete tree: 16 first-generation, 8 second-generation, 4 third-generation, 2 fourth-generation and 1 fifth-generation defeat. Further base enemies continue supplying repeat encounters.

Schema 3 adds `boss_generations: [[u32; 5]; 3]` without changing ordinary currencies, ranks or campaign fields. `boss_kills` remains the aggregate family count; `pending` remains the reserved first-generation flag. Missing or all-zero legacy generation vectors migrate prior aggregate boss defeats to generation one. Later counts clamp to half the preceding count, and the aggregate saturates at `u32::MAX`. Legacy base kills remain unchanged even when lower than prior boss defeats; reconstructed credit saturates at zero.

First-generation credit is `base_kills - generation1_defeats × 24 - reserved × 24`, with saturating arithmetic. A higher queue is `floor(previous_generation_defeats / 2) - current_generation_defeats`. The HUD exposes all five defeats and queue counts for each family. Queue counts include an active fight because it has not yet spent its credit.

Regular bosses grant 25/35/45/55/65 selected-ship XP, one pickup from each category, and the existing 25% signature-unlock chance with fourth-boss pity guarantee. Bounty progress counts each boss as one family kill. These rewards apply to production encounters only.

## Signature evolution

The highest actually defeated generation of a family evolves its signature, independently of purchased ranks. Rank damage and the selected ship's XP cap still apply. No additional save field is needed.

| Evolution | Pulse Echo | Repulsor Nova | Ion Wake |
| --- | --- | --- | --- |
| 1 | Three-shot piercing fan every eight shots | 70-unit blast | Two-second trail, 12 samples |
| 2 | Five-shot fan | 90-unit blast | 2.6-second trail, 16 samples |
| 3 | Adds two rear echoes | Clears hostile bullets inside the blast | Wider damage footprint |
| 4 | Pierces up to three targets | 120-unit blast and 15% shorter cooldown | 3.4-second trail, faster emission, 24 samples |
| 5 | Fires every six shots | Reflects up to 16 hostile shots and clears the remainder | Damage connects adjacent samples within 160 units, excluding sanctuary crossings |

Royale Sovereign has intrinsic evolution five in all three signatures, alongside its virtual rank-five upgrades. Winning the tournament does not require another ordinary-boss grind to complete that reward. Permanent global ranks, unlocks and ordinary defeat counters remain intact.

Reduced Motion freezes decorative extraction time, including reactor shimmer, cosmic drift, orbit markers and background twinkle. Actual attack charge, exposed-core state, movement, projectiles and simulation time continue. The player also stays visible during invulnerability instead of flashing.

## Validation and diagnostic isolation

`boss-dart-1` through `boss-dart-5`, `boss-bastion-1` through `boss-bastion-5`, and `boss-coil-1` through `boss-coil-5` create real diagnostic fights. The old unsuffixed names alias generation one. A transient diagnostic scene flag suppresses normal waves and all incidental progression, including stray shots destroying asteroids after the diagnostic boss dies. Exit through `home` or another scene command. Setup does not reserve a boss or grant qualification.

Portable native regressions cover legacy/invalid/saturated counters, the complete promotion tree, simultaneous family queues, one active boss, retreat/death/reload retries, duplicate callbacks, all 15 live patterns/phases, exact damaged rig extraction, core mitigation, selected-ship rank caps, Sovereign evolution, reflection/trail bounds, sanctuary guards and reduced-motion cues.

The independent public-API harness also defeated all 15 bosses using real joystick aiming, firing and pixel sweeps, observed each phase, confirmed unchanged diagnostic rewards, and verified a naturally queued fifth-generation fight across retreat/reload/hostile death/retry. Its declared level-12 Sovereign is a diagnostic loadout; that proves live mechanics and save isolation, not balance for a starting Scout. Evidence is in `work/boss-live-combat.json` and `work/boss-live-combat.log`. Browser and physical-device performance remain separate validation gates.
