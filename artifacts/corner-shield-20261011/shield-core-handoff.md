# Starter shield progression handoff

Frozen implementation: `362cd250677e39eb7dcd0ee678e26918615b111e` on `codex/starter-shield-progression`, based on roster commit `7be061bb387fac2c1a1155edabe4e25b42dd638a`. The tracked checkout is clean. Eleven committed files contain the approved core changes and focused test adjustments. No Root checkout, browser, package, save schema, currency, XP or inventory transaction was changed by this branch.

## Runtime contract

Normal chassis start with 30 shield HP: three chunks of 10. Effective Aegis hardware adds 10 HP per rank. Equipped Guard Coil retains its existing capacity of 20 HP at rank 1 and another 8 HP per later effective rank. Therefore fresh HUGO has 50 HP from its actual factory Guard R1; PAN, RENKO and an anonymous base chassis have 30 HP.

Recharge is locked until effective Aegis hardware R1 or equipped Guard R2. It retains the existing four-second no-hit delay. Its rank is `max(hardware_shield_rank, guard_rank - 1)`; the rate is `3 * (1 + .2 * (rank - 1)) * (1 + .03 * (min(ship_level, 12) - 1))` HP/s. The explicit enhanced Sovereign chassis adds 40 HP and retains at least 18 HP/s recharge. Hull repair remains independent at 0.4 HP/s per effective Repair Nanites rank, after five seconds without a hit.

The one additive serialized HUD field is `ship_stats.shield_recharge_unlocked: bool`. Existing `ship_stats.shield_regen` is the real rate, zero while locked. Profile remains schema 6 with identical stored ownership, ranks and XP. Empty slots remain allowed; no blueprint or equipped item is granted by this change.

A compatible, owned, nonzero effective item in the main corporation's matching slot enables PAN passive +10% speed/acceleration, RENKO offense +10% derived primary/signature damage, or HUGO defense 10% incoming damage reduction. Secondary manufacture stays visual. Clearing the slot removes the bonus. All checks use the actual selected hull and existing effective-rank/compatibility paths.

Capacity bounds proven with all ten owned hulls at maximum declared ranks are 132 HP for normal hulls and 172 HP for Sovereign, so the native shield display's 18-chunk limit covers the current catalog. Default Royal Aegis Sovereign has 120 HP. Factory training Guard R2 has 58 HP: five full chunks and one 8-HP chunk. A future legitimate capacity above 180 HP requires revisiting the UI bound.

## Executed checks

- Fourteen focused new core tests pass (296 filtered), using actual Game construction/HUD, public purchases/equip/switch/reload, real projectile hits and tick-based cooldown/delay behavior. Tests include all nine main/sub corporation pairs, all ten maximum-capacity hulls, actual emitted RENKO primary and champion projectiles, invalid/unequipped slots, and the hull0-only trained Guard R2 exception.
- All 310 `game_core` native tests pass, with zero failed/ignored tests; 16.74 seconds in the reported Windows debug test run.
- Strict native all-targets core Clippy and `wasm32-unknown-unknown` core Clippy pass with `-D warnings`. The latter is compilation evidence, not browser execution.
- `git diff --check` passed before commit.

The first whole-core run is retained as `core-first.log`: 297 passed and 11 failed against old capacity/stat expectations or hull/body fixtures newly protected by the base shield. Test-only repairs retain their original intent; hull/body probes explicitly deplete shields before actual damage. `player_presentation` now uses a real hurt path rather than a resetting diagnostic fixture. The final logs preserve the successful checks separately.

The training-upgrade test begins from a declared valid genuine-counter fixture and executes the real transaction; it does not claim a natural player completed the tutorial. The maximum-rank and Sovereign proofs similarly declare their authored checkpoint state. No human difficulty score, comfort judgement, normal dual-WASM package, GPU, phone or browser acceptance is claimed.

## Owned files and evidence

Runtime: `shield_progression.rs`; narrow `lib.rs` Stats/tick/hurt/HUD/module hunks; Guard description in `abilities.rs`.

Tests: `shield_progression_tests.rs`; approved test-only hunks in `identity_tests.rs`, `ship_progression_tests.rs`, `live_overlay_tests.rs`, `campaign_runtime.rs`, `biome_enemies.rs`, `radiation_enemy_tests.rs`, `player_presentation.rs`, and the existing dock repair test in `lib.rs`.

`source-pins.json` records all eleven physical source SHA256 values and committed Git blob IDs, plus successful logs and retained negative evidence. Git commit identity is authoritative for integration; Windows checkout newline bytes are pinned separately. Root can cherry-pick the single commit without copying any work-only output. Engine's separate shield UI commit is `dfe9312709c7ff831db8ecdf12bafc7212cb3079`; that peer handoff is not counted as our execution.

Pending: Root integration/build, actual browser capacity/locked-recharge presentation, and human balance evaluation of early capacity, recharge affordability and corporation bonuses. No further runtime scope is proposed in this handoff.
