# Next Scout playtest

Purpose: gather actual player evidence for first-flight pacing, Coil G1 and the relationship between XP and category points. No difficulty changes are proposed. Use one 15–20-minute desktop session, then repeat the relevant sections on real landscape touch hardware when available; synthetic touch is only a layout check.

Record the starting seed, ship, XP/level, basic/signature ranks, relics, hull/shield, input mode, viewport/zoom, browser and device. Keep natural progression and any declared fixture runs separate. Record incoming damage from hull/shield changes, not the generic hit sound/event, which also represents outgoing hits.

## Itinerary

| Step | Time budget | Player actions | Record |
| --- | --- | --- | --- |
| First flight | 2 minutes | Start with the fresh Scout at Haven. Turn, accelerate, release and stop; fly toward the belt. On desktop use click-course and right fire; on touch use joystick and fire button. | Time to belt, unintended thrust/turning, stop overshoot, first contact and remaining hull. Can the player understand drive power and firing direction without additional instruction? |
| Earn and return | 5 minutes | Break rocks, collect drops, enter first combat, then choose when to return to Haven. Do not mandate a kill quota. | Rock/base kills, XP and level changes, collected points per category, materials, hull on combat entry, expedition/death time, reason for returning. At the dock, record the first desired upgrade and whether XP, points or both prevent buying it. |
| Coil G1 | 6–9 minutes | Fight a naturally earned G1 if one is available. Otherwise use a separately labelled valid queued-Coil fixture: Scout, 24 Coil base kills, pending G1, 120 XP/L4, zero purchased hardware/relics/signature, radius tier one. Allow up to three attempts, first without coaching. Afterwards invite the player to try occupied head armor, exposed core and tail segments. | Outcome and encounter duration, first visible charge, first incoming damage, movement/fire conflicts, deliberate tail/core hits, missed shots, cause of loss/retreat and what the player learned. Preserve the raw first attempt separately from coached attempts. |
| Dock/death check | 2 minutes | Return normally or observe the next actual death, inspect the expedition report and dock. Try a purchase if affordable; otherwise identify the exact missing requirement. | Retained collected points/XP/materials, banking, ability to resume, whether the report explains progress, and the player's motivation to launch again. |

If the first boss is not reached within the budget, record that result; do not turn a forced diagnostic defeat into natural progression evidence. Explicit `boss-coil-1` diagnostics are useful for control practice but suppress rewards and normal waves, so they cannot validate XP/point pacing or production encounter pressure.

For a follow-up comparison, use separately declared Coil G1 fixtures with **one** rank-one purchase: Caliber, Shield or Speed. Each costs ten matching points; do not combine them into an assumed early loadout. Use identical seed, approach and input mode, and log the fixture grant. Contrast Dart/Bastion G1 afterwards if family-specific friction remains unclear. Small-view first-charge captures should include zoom 0.6 and 1.0; do not assume a zoomed-out screenshot proves readability at the normal view.

## Measurable questions and expected evidence

- **Drive and steering:** how long to reach the belt/combat, and how often does movement needed for survival point the primary gun away from the target? Expected evidence is a timed route and specific moments of control confusion or success, not an assumed ideal travel time.
- **Coil readability:** can the player identify charge, attack and exposed core, and find an occupied tail/head target? Capture the first charge and damage. The confirmed old immediate-tail contact must not recur; otherwise report the exact seed/angle/video as a regression. Whether an offscreen first charge feels fair remains a player question.
- **XP versus points:** what level is reached before the first affordable **desired** purchase, and how many minutes/returns/deaths precede it? Source consistency checks are +2 XP per destroyed rock, +5 per base enemy, +25 for a production G1 defeat, ten matching points per rank, and purchase gates at levels 1/3/5/8/12. Boss category drops require collection; uncollected loot is transient.
- **Recovery:** do collected points and XP survive actual death, do retained materials bank, and does the player understand what was gained? These are functional expectations. Their emotional value and whether repeat travel is tedious require the player's explanation.

Do not select a target win rate, buff amount or currency multiplier from this session alone. A useful first result is a complete recorded attempt plus an answer to “what made this loss or return happen?” If evidence points to a specific issue, compare a narrowly changed candidate in a later session with the same fixture and controls.

## Measured versus pending

Existing native evidence: 45 live isolated cases; exact-aim/no-dodge, no-purchased-hardware progression-floor bot won 9/15 and lost every Coil tier. Its Coil G1 loss occurred at 12.07 seconds, with first incoming damage at 2.60 seconds. It did not intentionally target tail armor and could miss hollow silhouettes. One fresh default-seed no-fire straight route reached belt/combat at 7.23/14.63 seconds after three asteroid contacts, leaving 76 hull. These are reference observations, not human performance targets.

Source deductions: a first same-Scout natural G1 has at least 120 XP/L4; mining, mixed kills and switching ships change that context. Before missions/boss drops, ten points in one specified category average 150 enemy kills or 120 rocks. Collected points can therefore lag XP gates. XP improves drive through level twelve but does not independently increase hull or primary damage.

Pending: human outcomes, mobile thumb precision/occlusion, natural concurrent-wave pressure, first-charge readability, actual time to the first desired upgrade, sustained physical-device performance and an entire natural progression playthrough. The full study and raw native results are in `work/scout-balance-review.md` and `work/scout-balance-results.json`; no new automated tests were run for this itinerary.
