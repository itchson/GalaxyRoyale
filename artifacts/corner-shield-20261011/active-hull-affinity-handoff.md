# Active hull manufacturer affinity repair

Frozen commit `f4188876c3792f18c2ae5848273fda2fff2fa43e`, parent `362cd250677e39eb7dcd0ee678e26918615b111e`, on `codex/starter-shield-progression`. Exactly two approved files change: `shield_progression.rs` and `shield_progression_tests.rs`. The tracked isolated checkout is clean. Prior shield source, handoff and independent vitals review remain unchanged.

The earlier helper selected the pilot's main corporation for every hull, causing an anonymous pilot to miss a crafted hull's bonus and a cross-corporation pilot to gain the wrong bonus. The new `main_manufacturer` resolves hull0 from its licensed factory identity, hulls1..8 from `CHAMPIONS[ship-1].identity.corporation`, and hull9 from the licensed main as Root's explicit current Sovereign policy. Sovereign has no separate authored manufacturer yet; its enhanced chassis and royal effects remain unchanged. Invalid hull IDs return no manufacturer. Neither sponsor nor sub-manufacturer chooses gameplay affinity.

The existing compatible/owned/nonzero equipped-category requirement remains. PAN passive drive+acceleration10%, RENKO offense primary/signature damage10%, and HUGO defense incoming damage reduction10% are unchanged numerically. Empty or invalid slots grant no affinity. No identity/hash, loadout, discovery, currency, XP, recipe, schema6, capacity, recharge, HUD or browser API changes are made by the runtime helper.

## Actual validation

Four new native tests use actual Game construction, HUD, public equip, real occupied-cell bullet hits and a real tick-emitted primary shot:

- All eight roster hulls with anonymous and PAN/RENKO/HUGO pilots:32 combinations. Actual effective speed/acceleration/damage and defense are tied to the authored hull main. Public matching-slot clearing removes its bonus; the stored license/hash and all XP remain unchanged.
- HUGO pilot on PAN hull4, whose sponsor/horns are HUGO:10 incoming damage with3 remaining shield causes7 hull loss, no false HUGO reduction. Anonymous HUGO hull6, sponsored RENKO:the same actual hit is reduced to9 and causes6 hull loss. Both erase real cells, emit one shield event and preserve the checkpoint. PAN pilot on RENKO hull2, sponsored PAN with PAN horns, emits one real14.52-damage primary rather than13.2 or a PAN drive bonus.
- Wrong-category/unowned defensive slots on HUGO hull6 are cleared by actual sanitation; no reduction applies to a real hit.
- Valid Sovereign fixtures retain the explicitly licensed-main fallback for each corporation, no affinity when anonymous,120 shield/18 HP/s recharge and zero fabricated ship XP.

`cargo test -p game_core shield_progression --locked`:18/18 pass,296 filtered;2.42 seconds in the recorded Windows debug run. `cargo test -p game_core --locked`:314/314 pass, zero ignored/filtered;7.53 seconds. Native all-targets and `wasm32-unknown-unknown` core strict Clippy pass with `-D warnings`. `git diff --check` passed before commit. Exact logs and source Git-byte hashes are in `manifest.json`.

The test profiles explicitly declare owned hulls/basics and victory where needed; they do not claim natural crafting, victories or difficulty acceptance. An initial test compile used the wrong Bullet field name (`friendly`); it was corrected to the actual `player` field before the recorded successful runs. No further runtime defect was found by those checks.

Engine's unchanged independent probe of the prior362cd25 source separately reproduced the original affinity defect. Its forthcoming repair rerun is independent peer evidence and is not claimed here until delivered. The frozen native vitals geometry review found no capacity/display blocker and remains pinned under `work/shield-vitals-independent-review`.

No Root source, package, shared manifest, public build, server, browser, push or user save was touched. Root owns integration and actual normal WebGPU/WebGL2 presentation; physical display and human balance acceptance remain pending.
