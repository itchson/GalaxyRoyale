# Champion campaign and expedition progression

The campaign has eight deterministic arena sites surrounding Haven. Their positions use the existing seed grid. Fly within 180 world units of the beacon and challenge its champion; descriptors appear within 320 units. Accepting a trial repairs the ship and activates combat at that arena only. Haven's 240-unit sanctuary remains protected. A 340-unit arena boundary contains the player, champion and its segments. Normal enemy waves and asteroid collisions are excluded from the arena.

| Champion | Ship | Encounter | Crafted ship signature |
| --- | --- | --- | --- |
| Lyra Voss, Human | Dawnneedle | Stutter dash and narrow triple burst | Two fast needle shots every four primary shots |
| Korr-9, Automaton | Bulwark | Rotating five-shot broadside | Side shots every six shots; +20 hull and -8% speed |
| Sable Myrr, Vesper | Nightrail | Sinusoidal serpent and rotating spiral | Guided missile every five shots |
| Xil of Aurora, Crystalline | Prism | Lateral weave and paired fork volley | Forked shots every three shots |
| Maera Quill, Human | Windrunner | Charge and damaging wake | Fast-flight wake; +10% speed and +15% acceleration |
| Orun Vale, Golem | Ironwake | Telegraphed ram and radial nova | Damage-triggered repulsor; +10 hull |
| Vexa Null, Voidborn | Voidcoil | Circling helix and inward volleys | Piercing seeker every four shots |
| Atlas-0, Ancient automaton | Sunspear | Long-range sniper burst | Fast piercing spear every six shots |

Defeating a champion through actual combat grants its patch and blueprint together, three matching category points and 40 XP to the selected ship. Rewards are issued once. The encounter uses real pixel masks, hull damage, projectile sweeps and destructible segments. It does not advance the ordinary boss queues. Champion ships have separately authored pixel outlines from `assets/ui/campaign-atlas.json`; their behavior builds on the three existing enemy families. Additional portraits, environments and encounter choreography can still be expanded.

At Haven, each blueprint costs ten matching category points and six stored materials to craft. These costs are checked together before either is deducted. Switching ships also requires Haven. Ship IDs are 0 for Haven Scout, 1–8 for the crafted champion ships, and 9 for Royale Sovereign.

## Galaxy Royale league

All eight patches unlock a distinct Galaxy Royale beacon on the seed grid at `0:2`. Fly within 220 units to start the next match. Each match takes place in the isolated arena around the beacon, and completion returns the player to it. A match starts with repaired hull and shield.

The league has nine entrants and 36 matches: eight actual player duels and 28 seeded NPC-versus-NPC outcomes. The table labels those outcomes as simulated. Each win awards three standings points. Ties use a stable seed-derived key, then entrant index. Death, returning home or retreating from an active match records a player loss. Once a losing league is complete, entering again begins a new attempt with cleared player results; the NPC outcomes remain stable for that seed.

Finishing first grants Royale Sovereign and the credits trigger once. Its victory record stays intact; post-victory exhibition leagues are not implemented. Sovereign has intrinsic rank-five hardware and all three existing signature abilities, stronger hull/shield/drive, and an eight-shot prism nova with repair. These are effective ship properties and do not replace the player's permanent ranks, unlocks or currencies. Its XP starts at level 12 when awarded.

## XP, cargo and run history

Each ship has separate XP. Ordinary enemies grant 5 XP, regular bosses grant 25/35/45/55/65 by generation, asteroids 2, champion trials 40 and winning league duels 30. Level 1 begins at zero XP; reaching level `L` costs cumulative `10 × (L - 1) × L` XP. Upgrade ranks require ship levels 1, 3, 5, 8 and 12. Six ordinary hardware ranks remain global and continue applying when switching ships; new purchases require the selected ship's level. The strength of the three acquired signature abilities is capped by that ship's current rank cap.

The Scout begins at 100 world units/second with 260 units/second² acceleration. Ship XP adds 2.5% speed and acceleration per level after level 1, capped at level 12, independently of the purchased 4%-per-rank drive upgrade. Crafted ships start from 180 speed and 400 acceleration before their family bonuses. Sovereign retains its intrinsic high-performance drive (248.4 speed) without another XP multiplier.

Category points bank immediately and survive death. Destroyed asteroids have an independent 25% chance to drop one material. The material hold carries at most 16; a full hold leaves uncollected materials in space. Returning to Haven, dying or reloading banks retained cargo into stored materials. Construction consumes stored materials.

An expedition begins when the ship leaves Haven. Returning or dying records its duration, peak distance, defeated enemies/asteroids/champions, and actual point/XP/material gains. The most recent completed summary persists in the profile. Reloading preserves that history without emitting a death event.

## Save and diagnostic boundaries

Schema 3 extends prior profiles with campaign state, per-ship XP, cargo, storage and the last run. Version 1/2 migrations preserve ordinary currency, ranks, unlocks and counters. Blueprints must match patches; crafting requires a blueprint; Sovereign ownership requires all patches, a complete league and first place in the seeded standings; credits acknowledgement requires that victory.

Live entities, current arena health and in-progress expedition timing are transient. Reloading returns to Haven. A league result is saved when a match finishes; an interrupted unfinished match remains unplayed. This is the solo checkpoint rule, not an authoritative competitive record.

Explicit `champion-0` through `champion-7` diagnostics create live fights for testing but never issue patches, blueprints, XP or match results. `tournament` moves to the beacon without granting qualification or wins. Production rewards always require the appropriate real encounter completion.

The native simulation suite verifies sanctuary overrides, actual pixel hits, containment, eight live movement/weapon patterns, once-only rewards, losses/retries, migrations, XP gates, cargo bounds, atomic costs and Sovereign's intrinsic upgrades. Browser and physical-device validation remain separate from native CPU proofs.
