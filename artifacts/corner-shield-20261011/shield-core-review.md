# Independent shield core review

Final verdict: no remaining scoped blocker in `f4188876c3792f18c2ae5848273fda2fff2fa43e`. The original `362cd250677e39eb7dcd0ee678e26918615b111e` had a verified active-hull manufacturer defect. The repair resolves all four failures in the unchanged independent public Game probe.

## Exact scope

Review completed 2026-10-11 UTC in the isolated `galaxy-hud-navigation` checkout. Each source snapshot contains 99 exact committed blobs: crate/workspace inputs and their actual included assets. The repaired snapshot differs in only `shield_progression.rs` and `shield_progression_tests.rs`; the other 97 inputs are byte-identical. Original snapshots and failure logs remain retained.

Reviewed the shield module, affected Stats/Hud/damage/tick paths, effective hardware/equipped rank gates, and constructor/loadout sanitation. Only `game_core` and its required `engine_core` dependency were compiled. Materialized `game_web` files satisfy the unchanged workspace manifest; this review did not build or test that crate, WASM, the renderer, browser, or deployed package. The checkout remains on `dfe9312709c7ff831db8ecdf12bafc7212cb3079`; neither reviewed core commit was cherry-picked.

## Verified defect and repair

Original `matching_slot` read the pilot's licensed corporation for every selected hull. Ships 1–8 have authoritative main manufacturers in `CHAMPIONS[ship-1].identity.corporation`. An anonymous pilot therefore lost that hull's bonus, while a licensed pilot could receive the wrong hull bonus. Sponsor and secondary manufacturer must not decide combat affinity.

| Public Game fixture | Original observation | Required result | Repaired result |
| --- | ---: | ---: | ---: |
| Anonymous PAN hull 1, equipped passive | top speed 187.2 | 205.92 | pass |
| Anonymous RENKO hull 2, equipped offense; PAN sponsor | primary damage 13.200001 | 14.520001 | pass |
| Anonymous HUGO hull 3, equipped Guard, actual diagnostic 32 damage | shield remaining 18 | 21.2 | pass |
| HUGO pilot on PAN hull 1, equipped Guard, actual diagnostic 32 damage | shield remaining 21.2 | 18 | pass |

The repair selects licensed main for hull 0, authored hull main for 1–8, and explicitly retains licensed main for hull 9 because Sovereign has no authored manufacturer. It changes no identity, designation, hash, save field, currency, XP, ownership, or shield formula. Source references in the repaired snapshot: `shield_progression.rs:46` manufacturer selection and `:66` compatible equipped-slot gate; `lib.rs:2272` actual incoming damage multiplier.

## Shield/progression findings

- Base capacity is 30 HP. Effective Aegis hardware adds 10 HP per rank. Equipped Guard Coil adds 20 at rank 1 and 8 per later effective rank. The explicit Sovereign chassis adds 40; virtual hardware remains separate. Normal maximum 132 and Sovereign maximum 172 fit the separately approved 18-mark presentation bound. No shield-count or browser claim is made here.
- Recharge remains locked at hardware 0 / Guard 1. Hardware 1 or equipped effective Guard 2 unlocks it. Rate follows effective rank and that ship's level; normal no-hit delay remains four seconds. Hull repair retains its separate five-second condition. `Hud.ship_stats.shield_regen` is the real rate and `shield_recharge_unlocked` is derived from it.
- Affinity requires an actual owned, compatible item in the manufacturer's canonical slot: PAN passive, RENKO offense, HUGO defense. The bonus does not apply to empty, unknown, unowned, incompatible, or wrong-category slots. Public constructor cases confirm malformed slots clear without capacity/recharge bonuses. Internally indexed helpers operate after the existing Game sanitation boundary.
- The public checkpoint probe preserves exact serialized profile equality, owned R5 hardware/Guard, XP, and currency through reload, with actual max shield 132 and regen 7.182. This is an authored checkpoint fixture, not a claim that those resources were earned during this probe.

## Actual independent execution

| Exact source | Focused existing shield tests | Unchanged independent public Game probe |
| --- | --- | --- |
| `362cd250677e39eb7dcd0ee678e26918615b111e` | 14 passed, 296 filtered | 2 passed / 4 failed; exit 101 |
| `f4188876c3792f18c2ae5848273fda2fff2fa43e` | 18 passed, 296 filtered | 6 passed / 0 failed; exit 0 |

Focused tests execute actual Game shot/hurt/tick, training, purchase, equip, switch and reload paths as authored by Gameplay. The independent six-case harness uses public Game/Profile/Hud/serialization and the existing public damage diagnostic; it calls no private helper or resource grant. The diagnostic exercises reset plus actual `hurt_player(32)` semantics, not a natural enemy encounter. Native debug CPU execution only; no performance measurement.

Commands, exact library/binary hashes, source inputs and raw logs are recorded in `manifest.json`. Toolchain: `rustc 1.97.0 (2d8144b78 2026-07-07)`, `cargo 1.97.0 (c980f4866 2026-06-30)` on Windows. One initial harness compile used nonexistent `ShipStatsHud.max_speed`; compiler identified `top_speed`, and only the harness was corrected before either six-case execution. Both historical error logs are retained. Successful linked executable runs are the probe evidence; the original `probe-compile-362cd25.log` is that initial compile error, not a successful compile log.

Gameplay separately reports 314 full core tests and native/WASM strict Clippy for the repair. Those owner checks were not rerun or attributed to this review. Geometric shield/vitals validation, render quality, both GPU backends, physical devices, and natural play remain separate gates.

## Evidence verification

Run `python work/shield-core-independent-review/verify.py` from this checkout to verify both 99-file snapshots against their exact Git blobs and the declared raw-byte output pins. It performs read-only verification; it does not build, execute test binaries, or change source. Evidence outputs are isolated under this review directory, with no production/core edits.
