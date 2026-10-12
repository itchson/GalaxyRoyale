# Co-op Authority Seams - Bounded Source Review (read-only)

Scope: `crates/game_core/src/lib.rs`, `crates/game_core/src/abilities.rs` only. Source inspection only; no networking exists or is claimed.

## Five concrete authority blockers for real host+7 WebRTC co-op

**1. Single hardcoded player entity, no peer identity.**
`Game` holds exactly one `player: Entity` field (lib.rs:549) alongside one `profile: Profile` (550). `InputFrame` (lib.rs:134-142: `joystick`, `destination`, `aim`, `thrust`, `fire`) carries no player id, tick sequence, or sender identity. `control(&mut self, input: &InputFrame, dt)` (lib.rs:1303, invoked from `tick` at 1205) applies the single input directly to `self.player`. There is no per-peer input map to route 8 distinct control streams; every input path (`fire` at 1438 reads `self.player`'s `Motion` via `self.world.get::<&Motion>(self.player).unwrap()`) is single-player by construction.

**2. Owner references are non-serializable ECS handles.**
Projectiles and body segments store `owner: Entity` (lib.rs:338), consumed in damage attribution (`damage_segment` 2221-2286 reads `self.world.get::<&Brain>(owner)`, `Actor`, `Motion`, then `kill_actor(owner, kind, p)` at 2281) and sweep queries (1962, 2031, 2113, 2562 `s.owner == entity`). Entity handles are local world indices; they cannot be sent over WebRTC as stable identities for remote-owned shots, and kill/credit attribution has no wire-stable owner id.

**3. Mutable PRNG inside the authoritative tick with no snapshot/replay seam.**
`Game.rng: u32` (lib.rs:572) is seeded from `profile.seed` (660) and advanced in `random()` (969-971: `wrapping_mul(1664525).wrapping_add(1013904223)`), consumed during tick-side decisions including fire/capacity paths (`fire` at 1438-1465, shared `MAX_BULLETS` cap refusal at 1461). `tick` (1172) takes `dt` as a raw float clamped by `dt.min(1.0/20.0)` (1181) with no fixed tick index. Without an input-locked tick numbering and RNG state snapshot per tick, non-host peers cannot deterministically replay or validate, and diverging `dt` silently diverges fire cadence.

**4. Reward claims mutate the save inline from gameplay paths, guarded only by local proximity.**
`interact_site` (lib.rs:1099) gates only on `progression_enabled()`, local `current_site()` proximity, and `profile.relics.contains(&site.id)` / `quest.completed` checks (1109-1122), then writes `profile.quests[index].completed`, `profile.currency[site.family]` (`saturating_add(3)` at 1137-1138), and `profile.relics.push(site.id)` (1139) in the same call. `update_exploration` (1053) likewise pushes discoveries from position (`self.position()` based, 1062-1081). Authority is "the machine that called the fn" — there is no transaction/verification boundary a host could use to validate a remote peer's claimed location or eligibility before persisting.

**5. Save lifecycle is fused into world construction; no authority split for host exit.**
`Game::new(mut profile: Profile)` (lib.rs:597) sanitizes/migrates the profile (598-633) and seeds the world from it (`let rng = profile.seed;` 660; `identity::player_body(&profile)` 646). Progression writes bump `profile.revision` via `changed()` (967) with no commit point. Existing round-trip is `serde_json::to_string(g.profile())` then `Game::new(from_str(...))` (tests at 4585-4586, 4638-4639). If the host exits between a `changed()` mutation and serialization, there is no defined persist/rollback boundary, and a reloaded host re-derives world state from the same seed, so reward state and world generation must stay consistent across exit with no double-claim.

## Minimal tests needed for local profile preservation on host exit

Following the existing patterns at lib.rs:4585-4602 and 4638-4648:

1. **Exit round-trip test:** tick N frames, claim via `interact_site`, drop the `Game`, `Game::new(serde_json::from_str(&saved))`; assert `currency`/`relics`/`quests.completed` and `revision` match the pre-exit profile (mirror 4600-4602).
2. **No-double-claim across reload:** after reload, second `interact_site` for the same site id returns the "already been claimed" `Err` arm (1110-1111, 1120) and does not `saturating_add` currency again.
3. **Unclaimed-claim rollback:** tick toward a quest, exit before claim, reload; assert `claim` still succeeds exactly once and `relics.len()` increased by exactly 1 (guards against persisting a phantom claim).
4. **Remote-claim isolation:** reuse the existing "objective completion must not claim remotely" style guard (4648) — `progress < target` on a reloaded host keeps `completed == false`.
5. **Revision monotonicity:** assert `profile.revision` is non-decreasing across save/load and that a rejected claim path (Err returns from `interact_site`) never bumps `revision` via `changed()` (967).

These are persistence-integrity tests only; they assert no networking behavior and presume no invented transport API.
