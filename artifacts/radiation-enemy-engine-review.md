# Isotope Mote: independent engine review

Reviewed the isolated `work/radiation-enemy/` prototype on 2026-10-10 UTC. Only this review was authored by the engine reviewer. No production integration, dependency addition, publish, authoring-script regeneration, browser acceptance or physical-device claim belongs to this pass.

## Result

No blocking source defect was found in the declared 40×40, uniform-scale 0.6 controller/art contract. The native suite independently passed **12/12**. This accepts a portable controller and occupied-cell concept for a later integration pass; it does not accept an actual production encounter, player damage, progression or reward flow.

The prototype's README now explicitly accounts for projectile commands reserved earlier in the same ECS tick. That clarification followed this review; the controller, atlas and tests were not changed. A separate evidence correction was sent to gameplay: the palette defines 15 distinct opaque colors, while the current occupied artwork uses 11.

## Geometry and rendering

The runtime atlas has 712 occupied cells in a 40×40 field. Its three emitter masks contain 142, 143 and 143 cells: **428 distinct cells with zero overlap**, all initially occupied. Direct JSON inspection gave an original-body conservative corner radius of approximately **13.8205 world units** at scale 0.6. This includes half-cell diagonal clearance; deriving the host radius from the complete authored body preserves a conservative radius as damage removes cells.

`Pose::transform` uses position, `heading + PI/2`, and uniform scale, matching production `actor_transform`. Emitter direction ranking uses a translation-free inverse transform. It chooses the nominal pylon using the pristine role geometry, then chooses an actual surviving cell from that same pylon. The final world origin uses the same body transform. A missing pylon never moves its shots to a different intact pylon. Arbitrary replacement dimensions or nonuniform scales are outside this asset contract; the host must retain the original 40×40 indexing and body-derived radius.

`draw` extracts the damaged body directly through `FrameBuilder`, so it cannot restore destroyed cells. It animates surviving palette cells, with light response 0.75 and emission 0; the warning pixels are unlit world-space presentation. There is one rigid body transform, not an independently articulated collision assembly. The intact body plus charge cue is bounded by **732 quads** (712 + 14 boundary dots + at most six lane dots). Palette indices use 11 distinct opaque colors, despite the 15 defined palette tones. The original authored PNG preview was inspected; it is art evidence only and is not loaded by the runtime.

The emitter mask includes the housing as well as its bright center. Disabling that sector requires the entire mask to become empty; erasing only the glowing center does not disable it. That rule is source-backed and should be included in later damage/readability tuning.

## Deterministic controller and spawn boundary

`Birth` fixes seed, actual spawn-region grid, immutable spawn-roll bits, biome, family and ordinary status. Its hash admits roughly one quarter of eligible family-0 births in biome 4, which current `regions.rs` names Radial Wastes. Production `spawn_enemy` assigns `Brain.phase` once from the existing RNG; the current ordinary AI reads it without mutation. Using those bits avoids an additional persistent serial or RNG draw. The future hook must evaluate the **actual spawn position**, as the current Riveter hook does, rather than substituting the player's current biome. Boss, champion, arena and explicit QA exclusion remain host responsibilities.

Valid positive steps are at most 0.05 seconds. Charge lasts 1.1 seconds, its final 0.35 seconds locks aim, two six-command pulses are separated by 0.35 seconds, attack lasts 0.5 seconds and recovery lasts 4.4 seconds. The fixed output array admits at most six commands per step. Consuming a pulse even when slots or emitters are unavailable prevents a deferred saturated burst. Zero/invalid/overflow inputs are tested without advancing controller state.

The warning-frustum check admits a charge when the complete hull and warning fit the current unshaken world viewport. It does **not** cancel an already admitted attack if subsequent camera motion or zoom hides the warning. That is an explicit pending readability/playtest boundary, not proof that every emitted pulse is preceded by an onscreen cue throughout its lifetime.

## Safety and host adapter gates

The prototype validates at most ten supplied sanctuary circles and separately honors the host's global player refuge. This covers a protected player outside the enemy's local site cache. The local guard separates the whole supplied body radius, removes inward velocity and suppresses a displaced step's firing. Its helper assumes the existing widely separated Haven/site geometry; it is not a general solver for overlapping arbitrary circles.

The prototype rejects a new shot crossing a supplied sanctuary during its initial step. **Every later projectile sweep remains the host's responsibility.** Current production collision code supplies Haven, arena-filtered nearby sites and distant-cache fallback; future dispatch must preserve that behavior, the normal four-second hostile lifetime, ordinary occupied-cell damage, shields/reflection, abilities and the shared projectile cap.

The later integration pass needs these focused gates:

- Install the variant once after the eligible ordinary Dart birth; retain `Kind::Enemy(0)`, family/tier hull and damage, and exclude it from generic AI so movement and shots are not generated twice.
- Replace body/scale/radius together; apply returned position, velocity and heading before the existing single physics integration, followed by the existing post-integration sanctuary guard. Use the exact interpolated presentation transform for rendering.
- Supply remaining capacity from the existing global cap, subtracting pending same-tick commands, or dispatch immediately before recounting. Existing `spawn_bullet(..., false, 1)` remains the final hard cap: 180 normally and 300 in explicit stress.
- Exercise `Game::tick` with a Mote at Haven and a remote site, including projectiles crossing a refuge on a later lifetime step and a protected player outside its local cache. The portable helper tests do not prove ECS adapter coverage.
- Exercise actual player hits, shield/invulnerability, reflection, ordinary once-only kill/XP/bounty/drop/boss-queue behavior and retirement. None is integrated by this prototype.
- Inspect the charge cue during movement, zoom and overlapping Motes in the browser. Measure actual extraction/frame time after integration. The 732-quad content bound is not phone FPS evidence.

## Independently run evidence

Toolchain: `rustc 1.97.0 (2d8144b78 2026-07-07)` on the current Windows workstation.

```powershell
cargo test --manifest-path work/radiation-enemy/Cargo.toml -- --nocapture
```

Exit 0; 12 passed, 0 failed; native test execution reported 0.12 seconds. The 2,400-tick / 40-second controller replay printed:

```text
40s native controller replay: 5 charges, 60 actual shot commands, <=6 per tick, <=732 extracted quads
```

The suite also checks actual Radial Wastes identity, deterministic selection/replay, aim lock, both pulses, actual muzzle cells, destroyed emitters, cap exhaustion without deferred replay, damaged extraction, charge admission, Haven/remote/global refuge guards, a through-circle segment, zero/invalid inputs and occupied-cell Scout geometry. Its Scout case is an engine hit/damage proof, not production player health or reward evidence. Gameplay reported strict native/WASM Clippy success; this review independently reran only the focused native suite.

Frozen runtime/test SHA-256 fingerprints:

| File under `work/radiation-enemy/` | SHA-256 |
| --- | --- |
| `src/lib.rs` | `2FED5D82EEA15267E4065C4B4A9B798676AA7BC3E4BF7628E75EA58BA82AE588` |
| `src/art.rs` | `90F06CC3CE3B06B3412B99CAB6AD8467AAABBA121971DDA6ECA7260DD8CC3EAB` |
| `src/tests.rs` | `691AA5A408932CC922687597B587F5A5BCD003D8A63B08E0B965EF64F6B41E7E` |
| `assets/mote-atlas.json` | `004C24EEFECCBE25D836DD8AD1FF00346F86879492C2BFF3FE828721DC838714` |

Production source and publishing were left to the root agent. This review does not change the pending integration or human tuning status.
