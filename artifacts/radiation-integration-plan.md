# Isotope Mote integration plan

This is a source-backed plan for the next integration pass, prepared against the current ordinary-enemy ECS and isolated `work/radiation-enemy/` prototype. No production source, manifest, save or deployment changed in this pass. The current production biome variant is the Iron Belt Riveter; the Radial Wastes Mote remains a prototype.

## Content and evidence boundary

Retain the proposed ordinary Dart identity: family 0, biome kind 4 (Radial Wastes), one-in-four immutable birth hash, original 40×40 body at scale 0.6, ordinary tier-scaled hull/contact damage and ordinary rewards. This adds ranged area control, not environmental radiation, exposure, a new damage type, currency, additional roots or persistence.

The unchanged isolated controller/art/tests have a previous independent **12/12 native pass**. Its 2,400-tick replay observed five charges, 60 shot commands, at most six commands per tick and at most 732 extracted quads. The atlas has 712 occupied cells, three disjoint emitter masks of 142/143/143 cells, and conservative corner radius approximately 13.8205 world units at scale 0.6. Fifteen opaque palette tones are defined; eleven are used by occupied artwork. These are prototype content/controller measurements, not current `Game::tick`, production rewards, GPU frame time or phone acceptance. No native suite was rerun for this planning-only pass.

Runtime/test fingerprints still match the reviewed prototype:

| Source | SHA-256 |
| --- | --- |
| `work/radiation-enemy/src/lib.rs` | `2FED5D82EEA15267E4065C4B4A9B798676AA7BC3E4BF7628E75EA58BA82AE588` |
| `work/radiation-enemy/src/art.rs` | `90F06CC3CE3B06B3412B99CAB6AD8467AAABBA121971DDA6ECA7260DD8CC3EAB` |
| `work/radiation-enemy/src/tests.rs` | `691AA5A408932CC922687597B587F5A5BCD003D8A63B08E0B965EF64F6B41E7E` |
| `work/radiation-enemy/assets/mote-atlas.json` | `004C24EEFECCBE25D836DD8AD1FF00346F86879492C2BFF3FE828721DC838714` |

## Exact production hooks

Use one internal Mote component/controller/art module, following the existing Riveter pattern. A sibling `crates/game_core/src/radiation_enemies.rs` is a small ownership boundary; no generic behavior framework is needed. Port the reviewed pure controller into production source and compile the indexed asset from `assets/pixels/mote-atlas.json`. Do not add a production dependency on the scratch crate or `work/` path. Existing engine/serde dependencies suffice.

| Current seam | Required change in the later pass |
| --- | --- |
| `Game::update_spawns` after `spawn_enemy(family, false, spawn, tier)` | Call an internal `maybe_mote(entity, regions::at(seed, spawn))`. It must reuse the existing entity and actual spawn biome/grid. |
| `Game::spawn_enemy` / `Brain.phase` | Read the already assigned, immutable `phase.to_bits()` as the ticket. Do not consume another RNG value, roll against the player's biome or reselect when crossing a biome border. |
| Variant installation | Require `Kind::Enemy(0)`, `Brain.family == 0`, `!Brain.boss`, no champion/boss/other variant component and no arena, reward-isolated QA or stress setup. Insert the Mote controller plus owned live body; update scale and conservative radius together. Preserve health, max health and ordinary contact damage. Set `previous_heading` consistently with any initial heading change. |
| `Game::tick` before generic AI / physics | Run `update_motes(player, dt)` after existing biome updates and before `update_enemies`/`integrate`. Snapshot global player refuge, current unshaken camera/world viewport and remaining shared projectile capacity before mutable ECS borrows. |
| `Game::update_enemies` query | Add an optional Mote component and skip installed Motes, alongside current champion/boss/Riveter skips. The component exclusively owns their AI; no generic extra movement or shot. |
| `Game::integrate` | Apply the controller's returned pose first, then let the existing integration move it exactly once. Keep `Game::enforce_actor_sanctuaries` after integration. |
| Shared shot dispatch | Dispatch each returned command through `spawn_bullet(position, velocity, damage, false, 1)`. Subtract earlier staged same-tick commands from remaining slots, or dispatch/recount immediately. Use the existing total 180 cap (300 in explicit stress) and four-second hostile lifetime. Consume dropped pulses; do not queue them for later. |
| `Game::render_into` actor query | Add optional Mote metadata and call `draw_mote` once, then continue. Use the already interpolated position and shortest heading, the current damaged body and actor scale. Include warning extent in this class's culling radius rather than only `actor.radius + 10`. |
| `Game::damage_actor` / `kill_actor` / `remove_enemy` | Keep the existing paths. Mote defeat is one ordinary family-0 defeat. Retreat is `remove_enemy(entity, false)` and grants no rewards. Component despawn happens with its entity. |
| Reset / campaign entry | Existing dock and arena combat cleanup finds all `Brain` entities; retain `Brain` so the Mote is removed there too. Live controller state stays transient. |

The portable input validates coordinates within ±1e6, while current `spawn_location` can place an enemy beyond the confined player's world boundary. The adapter must not install an out-of-domain Mote, and must handle a later invalid controller input without fire or a permanently frozen AI component. A deterministic ordinary retreat is a safe failure behavior; test the world edge rather than broadening the practical f32 extent silently.

## Camera-aware approach requirement

The fixed 180-world-unit orbit cannot be copied unchanged. The prototype admits charge only when `abs(enemy-camera) + actor.radius + 35 < world_viewport/2` on both axes. With a centered stationary player, the furthest admitted center is the diagonal of the remaining rectangle:

| Logical UI / zoom | Actual world viewport | Furthest admitted center (world units) | 180-unit orbit fits somewhere |
| --- | --- | --- | --- |
| 640×360 / 1.0 | 640×360 | 301.24 | Yes |
| 533×300 / 0.6 | 888.33×500 | 443.59 | Yes |
| 533×300 / 1.6 | 333.13×187.50 | 126.02 | **No** |
| 720×360 / 1.6 | 450×225 | 187.33 | Only limited angles |
| 400×240 / 1.6 | 250×150 | 80.55 | **No** |
| 320×180 / 1.6 | 200×112.50 | 51.72 | **No** |

These are exact geometric calculations from the existing bounds, not simulated or browser-observed production attacks. Browser `update_world_viewport` already uses current eased zoom before fixed ticks; the controller should use that extent and `Game::render_camera(1.0)`, without presentation shake.

Adopt a bounded approach-and-hold policy, keeping the complete warning gate strict:

1. Define a camera-centered firing rectangle with half extents `world_viewport/2 - (actor.radius + WARNING_REACH + 2 world units)`. The extra two units give clearance from the strict warning boundary. A nonpositive extent means no firing center exists at this reach.
2. While Waiting, try at most five targets: the preferred player-relative 180-unit point clamped into this rectangle, plus its four corners in a stable birth/orbit-sign tie order. Use the actual player body radius, not a Scout constant. Reject a target or direct approach segment that enters `player.radius + actor.radius + 8` clearance or a supplied sanctuary expanded by the Mote's radius and existing six-unit body guard. Choose the closest legal target to the current Mote position; retain it while legal to avoid corner chatter.
3. Approach that target with bounded velocity and steering, using existing maximum movement/tier ceilings. Settle before charging: damp arrival velocity and require current and predicted next-step complete-warning clearance. Do not teleport the entity, alter the player's camera/zoom or start the cue offscreen.
4. During Charge and Attack, hold the readable target instead of applying the old 180-unit outward approach/strafe. That old Charge motion would push a newly admitted small-view Mote back outside its narrow firing rectangle. Recheck current and predicted next-step complete-warning bounds using current camera and eased viewport. Loss of clearance aborts the warning and returns to Waiting/repositioning before emitting another pulse.
5. Recovery may resume the bounded approach/orbit. At an extent or sanctuary configuration with no collision-clear target, stay nonfiring and retreat rather than weakening the warning gate. The later pass must explicitly document and test any genuinely no-fit extent. If a declared supported view cannot provide a readable encounter, revise the bounded cue/approach design before acceptance; do not count indefinite Waiting as success.

This policy is a proposed production adaptation, not implemented prototype behavior. New state/input for actual player radius and a retained approach target needs focused native tests. A small fixed candidate set avoids navigation meshes and unbounded per-frame searches. Path rejection may temporarily find no target around an obstruction; it must fail without unseen shots and retry as geometry changes.

## Occupancy, collisions and safety

Rendering, collision and shot origins must share the same owned 40×40 `PixelBody`. Retain the transparent reactor opening, stable centered cell coordinates and damage bitset. No freshly cloned pristine body may replace the damaged body during phase/render changes. Every emitted origin is a surviving cell in its nominal emitter mask transformed by the same `heading + PI/2` uniform-scale transform. Destroying the entire pylon mask disables its sector without a replacement from another pylon; destroying only its bright center does not currently disable it. Use the declared rule or explicitly redesign and retest it.

Existing bullet collision first performs a conservative circle check, then swept `PixelBody::hit_segment` in object-local coordinates, and selects the nearest occupied contact. Keep that path so shots can traverse real holes and fast projectiles cannot tunnel through occupied cells. A Mote never needs a separate bounding-circle damage implementation or area-damage/radiation collision overlay. Retain ordinary hull contact, shield absorption, 0.45-second player invulnerability, Repulsor reflection/clear, Pulse Echo and Ion Wake.

Use Haven plus the relevant arena-filtered seeded 3×3 site network, bounded at ten circles, and the global `safe_zone_at(player)` result. A player may be protected beyond the enemy-local cache. For actor and projectile checks outside the current cache, retain the host's distant lookup fallback. The prototype guards a shot's first step only; current `resolve_collisions` must continue dissipating every later swept segment through Haven or sites throughout its lifetime. Preserve the post-integration actor guard and remove inward boundary velocity. Decorative gas, flares and parallax radiation imagery never define gameplay safety or damage.

## Rewards and persistence

The current ordinary defeat path gives five selected-ship XP, increments `profile.kills[0]` and family-0 kill credit, advances an accepted matching bounty once, performs the existing 20% category-point drop roll and can reserve the existing first boss at 24 family kills. Currency is earned by collecting the ordinary drop; defeating the Mote must not directly credit points or crafting materials. Unrelated boss generations, unlocks and currencies must remain unaffected.

`kill_actor` begins with an entity-existence check, so a second call after despawn cannot grant another reward. `remove_enemy(..., false)` grants none. Reward-isolated diagnostics currently use `boss_qa_scene` to suppress ordinary rewards/exploration; a proposed `SCENE biome-mote` must follow that route and be visibly QA-only. Native normal-play reward fixtures must not use the QA flag. There is no new saved controller, variant field, counter or schema version; reload removes transient live enemies and keeps the existing profile progression.

## Meaningful acceptance tests for the later pass

The following are required new production tests, not claims of passing evidence:

- **Actual birth hook:** deterministic eligible/ineligible actual spawn-biome samples; hostile spawn on a biome boundary; ordinary Dart only; no arena/champion/boss/other variant/QA/stress installation; no extra entity or RNG draw; fixed body and birth identity across a border.
- **Single authoritative update:** installed Mote produces the intended two pulses through real `Game::tick`, receives no generic AI shot, integrates movement once and keeps prior transform history valid. Validate dt 1/60 and 0.05, pause, zero/invalid input and invalid-geometry failure behavior.
- **Camera-aware encounter:** stationary player and naturally approaching/orbiting Mote at 533×300, 720×360, 400×240 and 320×180 with zoom 0.6/1.0/1.6 world extents. Each geometrically feasible fixture must approach, settle and charge in bounded simulated time. During camera lag, player movement, zoom easing and viewport changes, no new pulse may follow an incomplete/unseen warning. Test each no-fit policy explicitly; do not relax the visibility gate.
- **Exact live cells:** rotated/far-world shot origins remain in surviving nominal masks; erase a selected outer origin then observe a new live origin; erase one/all pylon masks and verify only the owned sectors disappear, with no deferred pulse. Shoot through the reactor opening and an impact-created hole, then at an occupied cell through the actual shared collision path. Rendering must preserve those exact holes at alpha 0/0.5/1 and across angle wrap.
- **Shared cap:** several Motes and existing hostile/player fire with 0/1/5/6 available slots; account for same-tick reservations and the combined dispatch order. The total ECS bullet count must stay ≤180 normally and ≤300 in explicit stress, and freeing capacity later must not replay a dropped pulse.
- **Sanctuaries:** Haven and real remote sites, a protected player outside the enemy-local cache, a body/impulse approaching a circle at dt 0.05, and projectile crossings on both initial and later lifetime steps with endpoints outside the circle. Include distant-cache fallback and arena-filtered sites without spawning Motes in an arena.
- **Real player consequences:** an exposed moving Scout is hit by an actual hostile projectile; assert shield/hull/invulnerability and occupied-cell damage, reflection and abilities. A stationary Scout in the indicated center gap is intentionally safe. Two pulses are 0.35 seconds apart, shorter than current 0.45-second invulnerability: do not assume every projectile command causes another damage event.
- **Once-only ordinary progression:** defeat a non-QA Mote with real player shots at 23 family-0 kill credit; assert exactly one family kill, five XP, one matching bounty increment and one first-boss reservation, no unrelated progression and no direct currency credit. Repeat after despawn, test retirement without reward, bounded ordinary loot, profile round trip and all cleanup routes. Separately prove the QA scene cannot grant progression.
- **Presentation and capacity:** extract damaged/charging bodies repeatedly into the existing reusable frame; check finite records and valid transform slots, material response 0.75/emission 0, ≤live cells+20 cue quads, unchanged simulation under Reduced Motion/cosmetic density, and warning culling near a screen edge. Record native CPU timings against a fixed ordinary baseline, then inspect real visible WebGPU/WebGL2 captures at supported zooms. Physical mobile FPS/readability/input acceptance remains a separate gate.

At the normal 48-enemy root cap, even a hypothetical all-Mote scene has at most 35,136 body-and-warning quads before other world content. That is a content bound, not a frame-time result; collision cost and existing backgrounds/asteroids/effects still need measurement. No additional light list or shader is required: the existing material and capped projectile lights already accept the class.

## Next-pass ownership and verification

Gameplay owns the production component/controller, spawn/update queries, collision/reward tests and source docs. Root owns shared integration, diagnostics/browser bridge, visible captures, builds and publishing. Art/engine review verifies the indexed asset, occupied cells, transforms and extraction; independent QA checks the actual fixtures. Keep the dashboard record append-only and label each result by what actually ran.

After implementation, run focused Mote tests and strict native/WASM Clippy, then the required complete game checks for the changed integration seams. Capture the actual source/asset hashes, viewport/zoom settings, native replay/cap/reward outcomes and visible browser screenshots. Prototype tests and this plan cannot substitute for those records.
