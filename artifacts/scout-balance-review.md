# Starting Scout balance review

Review date: 11 October 2026. Production tuning is unchanged. This separates source deductions, actual native simulation observations, and proposed human/browser fixtures. The only production change during this review fixes confirmed Coil spawn contact; it does not adjust difficulty.

## Observed fixtures and limits

The scratch public-API harness in `work/scout-balance/` ran 45 isolated real encounters against the final outward-tail spawn fix: fifteen stationary fresh Scouts, fifteen exact-aim/firing fresh Scouts, and fifteen exact-aim/firing Scouts at their same-ship progression floor. Evidence: `work/scout-balance-results.json` and `.log`. Reproduce from the repository root with `cargo run --manifest-path work/scout-balance/Cargo.toml --target-dir target`.

Every case uses the default seed, a 533×300 **world** viewport, 60 Hz steps and a 60-second budget. The aiming bot turns toward actual `BossHud.position`, targets `core_position` during recovery, holds fire and pursues at joystick magnitude 0.65 beyond 420 units; otherwise it uses 0.081 to turn with little thrust. It has no dodge logic. Diagnostic scenes suppress ordinary waves and rewards. These are live projectile/pixel-hit observations, not human skill, browser framing, mobile precision or difficulty acceptance. The generic `hit` event also represents outgoing projectile hits, so incoming damage was measured from an actual decrease in Scout hull, plus actual death events.

The same-Scout floor fixture has no purchased basic hardware or relic hull bonus. It includes the family's guaranteed rank-one signature after four prior boss wins, with its derived evolution. This is a deliberately weak but legal spending choice, not an expected player loadout. A level-one Scout meeting generations two through five is an out-of-order stress fixture; normal same-Scout progression reaches those tiers at higher levels.

| Boss | Same-Scout floor | Actual outcome, seconds | Lowest observed hull before death/reset | First incoming hull damage, seconds |
| --- | --- | --- | --- | --- |
| Dart G1 | L4, 120 XP | Defeat, 8.17 | 87.6 | 7.98 |
| Dart G2 | L5, 290 XP | Defeat, 8.62 | 57.2 | 2.02 |
| Dart G3 | L8, 650 XP | Defeat, 19.62 | 100 | None |
| Dart G4 | L12, 1,390 XP | Defeat, 21.83 | 10.1 | 2.50 |
| Dart G5 | L17, 2,890 XP | Defeat, 15.20 | 20.6 | 2.05 |
| Bastion G1 | L4, 120 XP | Defeat, 12.58 | 13.2 | 2.33 |
| Bastion G2 | L5, 290 XP | Defeat, 15.82 | 100 | None |
| Bastion G3 | L8, 650 XP | Defeat, 20.90 | 83.9 | 4.60 |
| Bastion G4 | L12, 1,390 XP | Defeat, 26.18 | 82.0 | 12.37 |
| Bastion G5 | L17, 2,890 XP | Scout death, 20.33 | 0.8 | 2.02 |
| Coil G1 | L4, 120 XP | Scout death, 12.07 | 10.4 | 2.60 |
| Coil G2 | L5, 290 XP | Scout death, 15.05 | 11.2 | 2.40 |
| Coil G3 | L8, 650 XP | Scout death, 10.33 | 3.3 | 2.55 |
| Coil G4 | L12, 1,390 XP | Scout death, 8.58 | 10.1 | 2.18 |
| Coil G5 | L17, 2,890 XP | Scout death, 10.12 | 16.2 | 3.03 |

That bot defeated 9/15 floor encounters. It defeated 9/15 fresh L1 encounters too, but the sets differ: fresh L1 defeated Coil G2 with only 2.48 hull left, and lost Dart G5. This sensitivity is a reason to gather human play evidence, not to rank the tiers by one stationary strategy. Coil G3–G5 remained at roughly 75–77% boss hull when the floor bot died; its center aiming can miss intentional hollow silhouettes, and it never deliberately attacks tail armor. Players can choose occupied armor/tail targets, so that bot is especially limited for Coils.

Stationary, unfired fresh Scouts died against Coils G1–G5 after 11.45/14.38/9.28/8.15/9.15 seconds. Their first actual incoming damage occurred 2.10–2.98 seconds after setup, rather than immediately. Dart G3 and Bastion G2 did no damage to that fixed stationary position within 60 seconds; Bastion G3 disengaged after 38.25 seconds. These demonstrate aim-position and pursuit dependence; they do not prove exploits across mobile input, angles or concurrent waves.

A separate actual default-seed launch with full-right joystick and no firing reached the belt at 7.23 seconds and the first combat radius at 14.63 seconds. It took three asteroid contacts and arrived with 76/100 hull. Its peak measured translational speed was approximately 100 world units/s. This is one straight-line seed path, not a median onboarding experience or a suggested travel-time target.

The independent durable `scripts/qa/boss-live` probe still defeated all fifteen bosses after this fix using its declared fully upgraded Royale Sovereign. That validates encounters and diagnostic isolation, not starting-Scout balance.

## Source deductions

The initial Scout has 100 hull, zero shield, 12 primary damage, five shots/s, speed 100 and acceleration 260. Its occupied-cell conservative contact radius is 19.90 world units. XP increases drive by 2.5% per level up to level twelve, without increasing hull, shield or primary damage. Purchased basic ranks remain global; ship XP gates the **next purchase**, while signature rank strength also observes the selected ship's level cap.

| Ship level | Total XP threshold | No-purchase speed / acceleration | Maximum purchasable rank |
| --- | --- | --- | --- |
| 1 | 0 | 100 / 260 | 1 |
| 3 | 60 | 105 / 273 | 2 |
| 4 | 120 | 107.5 / 279.5 | 2 |
| 5 | 200 | 110 / 286 | 3 |
| 8 | 560 | 117.5 / 305.5 | 4 |
| 12 | 1,320 | 127.5 / 331.5 | 5 |

At speed rank five, level-twelve Scout drive reaches 153 / 397.8. XP levels continue beyond twelve, but drive and the five-rank purchase gate stop improving. A newly crafted ship has its own XP; inherited global basic hardware still works, while signature rank strength can fall to that new ship's low-level cap.

Twenty-four same-family base kills give at least 120 XP on the same Scout, so its first natural G1 boss is usually **not** a fresh L1 encounter. Restricting all prerequisite kills/wins to that ship yields the table's G1–G5 floor XP of 120/290/650/1,390/2,890. Mixed-family kills, rocks and other encounters increase it. Switching ships or loading an old pending queue breaks that same-ship floor assumption.

The nominal unupgraded primary throughput is 60 hull/s before misses and mitigation. Closed boss armor reduces it to 33; a recovery core hit raises that shot's damage to 165%. Tier-one G1 hull is 270/432/540 for Dart/Bastion/Coil. Thus armor-only continuous-hit lower bounds are 8.18/13.09/16.36 seconds, without travel, missed cells, firing cadence boundaries, phase changes, segments or impulses. The real bot timings differ because it uses recovery cores, and collisions/holes affect its hits. These arithmetic bounds should not become encounter-time targets.

At radius tier one, a G1 projectile inflicts 12.4 hull damage, head contact 20 and a boss tail contact 13; invulnerability lasts 0.45 seconds. G5 projectile/head damage reaches 19.84/32. Armor rank one adds only 15 hull; a first shield purchase adds 20 shield and recharges after four undamaged seconds. XP drive alone therefore does not create additional mistakes a player can survive.

Points are the likely early progression bottleneck. Each base enemy has a 20% category-drop chance, uniformly split three ways; each rock has a 25% category-drop chance. Ignoring missions and bosses, one specified category's ten-point purchase takes **150 enemy kills or 120 rock kills on average**. Twenty-four base kills supply only 1.6 expected points in each category, although drops are stochastic and collection is required. Asteroids give two XP and independently have a 25% material-drop chance, so hull/drive XP gates can unlock well before enough points accumulate. Courier/bounty rewards supply three matching points each and can materially change that pace. Retained points/materials on death reduce lost progress, but a return trip or repeated deaths may still feel slow before the first installed upgrade.

The joystick couples thrust direction and weapon facing. PC click-course control also faces the destination. A Scout cannot continuously retreat and fire backward with its primary gun. Coil G1's attack velocity can reach roughly 131 world units/s and higher tiers exceed that; baseline Scout speed is 100–127.5 before speed purchases. Human weaving, attack timing and tail targeting therefore matter more than straight-line kiting. The bot's precisely maintained 0.081 stick is an artificial aiming convenience, particularly on touch.

Natural bosses appear outside the viewport's circumscribed radius plus 45 units. Their charge clock starts immediately, including while the head is outside the visible area. A tall/narrow spawn angle can keep the physical reactor/aim marker offscreen for its first charge; the boss HUD can still say charging. This is a source-based framing concern requiring browser observation, rather than evidence that every first attack is unreadable. Higher queued generations also get global priority: returning from an unfinished hard boss can produce the same boss again instead of a lower encounter, although retry credit is correctly retained.

## Coil framing and the confirmed spawn fix

At a 533×300 world viewport, a natural boss's head spawn distance is 350.81. Authored Coil head radii are 36.41/45.60/45.63/61.19/76.41; the initial head-to-last-segment spans are 175.02/289.98/359.90/482.69/604.51. The old fixed +Y placement from an above-player natural head therefore crossed the Scout's contact circle in G3–G5. The portable regression reconstructs the old G3 layout and observes **13 actual Scout hull lost in the first tick while its boss is still charging**, before attack/projectiles.

Production now initializes base Coils and all ordinary tiers outward from the head, away from the actual player. All initial head/segment headings face toward the player; previous headings match and the initial Coil aim agrees. A coincident position uses deterministic +Y fallback. A separate regression covers sixteen angles × two small viewports × base Coil and generations one through five, with no first-tick contact. All 63 core tests and strict all-target Clippy passed after that fix.

The diagnostic head's center remains 210 units above the player. At a 300-unit-high world view and zoom one, its center is 60 units above the top edge. All five down-facing Coil heads initially remain entirely above that edge: even the full 96-cell G5 square scaled 1.24 reaches only 59.52 units below its center; its actual occupied cells end at world Y −152.96 relative to the Scout. The conservative radius can make an extraction batch present even while its actual pixels are offscreen. At zoom 0.6 the corresponding world half-height is 250 and the center is visible. **That exact 210-unit placement is diagnostic-only.** Natural heads also intentionally spawn offscreen, but their angle and viewport-dependent distance differ. Initial tail crossing was not diagnostic-only; it is now fixed. Zooming to a small world view can still show only part of a long chain after normal entry, which is a framing/playtest question rather than another initialization collision.

## Concrete measurement plan before tuning

1. **Onboarding, real progression:** use fresh default profiles at three seeds, on desktop and landscape touch. Run five ten-minute expeditions per input mode without fixture grants. Record launch-to-belt/combat time, hull on first combat entry, collected points per category, time to first dock purchase, cargo/stash, return/death count and time spent travelling. Compare those results with the one-path observation above; no success percentage is accepted yet.
2. **First natural bosses:** use explicit valid fixtures with 24 family kills, a reserved G1, 120 Scout XP, zero hardware, no relics and no signature, plus separately declared single purchases (damage, shield or speed rank one). Test all three families at natural cardinal/diagonal spawns with ordinary waves enabled. Five human attempts per family/input mode should record first visible charge, reaction time to first incoming hit, attempts-to-win, missed-shot rate, tail/core targeting and retreat reason. Label saved fixture grants clearly; never treat them as earned play progression.
3. **Tier progression:** test G2 at XP290, G3 at650, G4 at1,390 and G5 at2,890 using their valid predecessor counters and mandatory unlocked signature, first with no hardware and then with a declared affordable loadout drawn from real onboarding medians. Record damage opportunities per recovery phase, phase-specific damage intake, chase time and victory/loss/retreat. Include alternate-ship low XP with inherited hardware as a separate fixture, not as a starting Scout.
4. **Framing/controls:** repeat G1 and G3/G5 Coil at physical landscape 300/360-height logical views and zoom0.6/1.0/1.6. Capture full first-charge video, head visibility, nearest segment distance, UI-covered threat area, camera lag and joystick precision. Include an actual mobile thumb occluding each control. The native world-extent fixtures are not substitutes for this observation.
5. **Deterministic pattern coverage:** replay each case at eight approach angles and several phase offsets with stationary, orbiting and retreat/re-engage inputs. Specifically investigate stationary no-damage observations for Dart G3/Bastion G2 and the retreating Bastion G3. Keep source/harness seeds, commands, outcomes and raw observations together. A dodge bot can compare mechanics repeatably, but it should not supply a human balance verdict.

Priority for the next human session: first-run upgrade pacing, Coil G1 with a legal early Scout, combined movement/fire precision, and the first offscreen charge. No difficulty changes are recommended without those observations.
