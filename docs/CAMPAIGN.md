# Champion campaign and expedition progression

The campaign has eight deterministic arena sites surrounding Haven. Their positions use the existing seed grid. Fly within 180 world units of the beacon and challenge its champion; descriptors appear within 320 units. Accepting a trial repairs the ship and activates combat at that arena only. Haven's 240-unit sanctuary remains protected. A 340-unit arena boundary contains the player, champion and its segments. Normal enemy waves and asteroid collisions are excluded from the arena.

| Champion                   | Ship       | Encounter                              | Craftable ability blueprint                    |
| -------------------------- | ---------- | -------------------------------------- | ---------------------------------------------- |
| Lyra Voss, Human           | Dawnneedle | Stutter dash and narrow triple burst   | Two fast needle shots every four primary shots |
| Korr-9, Automaton          | Bulwark    | Rotating five-shot broadside           | Broadside: side shots every six primary shots  |
| Sable Myrr, Vesper         | Nightrail  | Sinusoidal serpent and rotating spiral | Guided missile every five shots                |
| Xil of Aurora, Crystalline | Prism      | Lateral weave and paired fork volley   | Forked shots every three shots                 |
| Maera Quill, Human         | Windrunner | Charge and damaging wake               | Windrunner Wake: fast-flight damaging trail    |
| Orun Vale, Golem           | Ironwake   | Telegraphed ram and radial nova        | Impact Nova: damage-triggered repulsor         |
| Vexa Null, Voidborn        | Voidcoil   | Circling helix and inward volleys      | Piercing seeker every four shots               |
| Atlas-0, Ancient automaton | Sunspear   | Long-range sniper burst                | Fast piercing spear every six shots            |

Defeating a champion through actual combat grants its patch, hull blueprint and corresponding ability blueprint together, three matching construction-category points and 40 XP to the selected ship. Rewards are issued once. The encounter uses real pixel masks, hull damage, projectile sweeps and destructible segments. It does not advance ordinary boss queues. Champion ships have separately authored pixel outlines from `assets/ui/campaign-atlas.json`; their behavior builds on the three existing enemy families. Additional portraits, environments and encounter choreography can still be expanded.

At physical Haven, each hull blueprint costs 10 matching construction-category points and 6 banked Tier 1 materials. Costs are checked atomically before either is deducted. Hull construction and ability crafting are separate operations: neither crafting operation silently selects a hull or equips an ability. Switching ships also requires Haven. Ship IDs are 0 for Haven Scout, 1–8 for crafted champion hulls and 9 for Royale Sovereign. New hulls start at XP 0 / LV 1 and keep their own hardware/slots. Bulwark's +20 hull/−8% speed, Windrunner's +10% speed/+15% acceleration and Ironwake's +10 hull are chassis properties; their automatic weapon/trail/nova effects require equipped abilities.

## Forge inventory and per-ship loadouts

Every ship has exactly three automatic ability slots: offensive, defensive and passive. Six hardware ranks (damage, fire rate, hull, shield, regeneration and drive) are separate ship-local stats, not extra ability slots. Owned ability ranks are shared inventory; each hull stores its own compatible slot selection. Replacing a slot retains the displaced owned ability. Unequipped abilities have no hidden effects. Normal effective ability and hardware ranks are `min(owned_rank, active_ship_rank_cap)`; stored high-rank items remain owned at a lower level with reduced effective power. Forge discloses OWNED RANK versus ACTIVE RANK when they differ. Sovereign's royal/chassis exceptions remain explicit below.

The stable catalog has 17 entries:

| Slot      | Starter    | Ordinary boss blueprint | Champion blueprints                                                                 | Royale bundle |
| --------- | ---------- | ----------------------- | ----------------------------------------------------------------------------------- | ------------- |
| Offensive | Pulse Lens | Pulse Echo              | Needle Burst, Broadside, Guided Missile, Fork Volley, Piercing Seeker, Sniper Spear | Royal Prism   |
| Defensive | Guard Coil | Repulsor                | Impact Nova                                                                         | Royal Aegis   |
| Passive   | Flux Drive | Ion Wake                | Windrunner Wake                                                                     | Royal Wake    |

Starter blueprints are available from the beginning, and starter/boss abilities fit all owned hulls. Champion abilities fit their authored hull family: Scout/Dawnneedle/Prism/Windrunner are Dart, Bulwark/Ironwake/Sunspear are Bastion, and Nightrail/Voidcoil are Coil. Sovereign accepts ordinary abilities from every family; royal bundles are Sovereign-only. Construction currency and ability slot category are separate: Bulwark's Broadside is offensive despite its defensive hull-construction currency.

Crafting a starter costs 3 matching points and 2 Tier 1 banked materials; crafting other ordinary/champion abilities costs 10 points and 2 Tier 1. An ability upgrade costs 10 matching points and 2 materials of the next rank's tier. Each hardware upgrade costs 10 matching points and 1 material of the next rank's tier. Purchased ranks 1..5 require active ship levels 1/3/5/8/12. All costs/gates/compatibility are validated atomically in Rust. Royal bundles are victory rewards, not construction recipes.

The dedicated Forge opens with three slot summaries followed by the six hardware rows. Manage opens one category; Back returns to the overview. Craft acquires ownership, and Equip/Replace explicitly changes that ship's slot. The Hangar displays each owned hull's XP and selection/construction actions. HUD availability, prices and error reasons are authoritative; the shell does not recompute affordability or infer categories from ship names.

## Galaxy Royale league

All eight patches unlock a distinct Galaxy Royale beacon on the seed grid at `0:2`. Fly within 220 units to start the next match. Each match takes place in the isolated arena around the beacon, and completion returns the player to it. A match starts with repaired hull and shield.

The league has nine entrants and 36 matches: eight actual player duels and 28 seeded NPC-versus-NPC outcomes. The table labels those outcomes as simulated. Each win awards three standings points. Ties use a stable seed-derived key, then entrant index. Death, returning home or retreating from an active match records a player loss. Once a losing league is complete, entering again begins a new attempt with cleared player results; the NPC outcomes remain stable for that seed.

Finishing first grants Royale Sovereign and the credits trigger once. Its victory record stays intact; post-victory exhibition leagues are not implemented. A newly awarded Sovereign starts at XP 0 / LV 1, with enhanced chassis/hardware and three explicitly owned/equipped rank-five royal bundles. Royal Prism supplies the eighth-primary-shot prism volley plus evolved Echo; Royal Aegis supplies the eighth-shot 4-hull repair plus evolved Nova; Royal Wake supplies evolved trail. These bundles stay effective rank 5 regardless of XP while equipped. Removing or replacing a bundle removes its effects. Sovereign's stronger chassis and virtual rank-five hardware remain separate reward properties. Existing recorded XP is preserved by migration rather than replaced with a fabricated starting level.

## XP, cargo and run history

XP belongs to individual ships, never a player-level pool. Ordinary enemies grant 5 XP, regular bosses 25/35/45/55/65 by generation, non-belt asteroids 2, champion trials 40 and winning league duels 30. Belt asteroids grant no XP. Level 1 begins at zero XP; reaching level `L` costs cumulative `10 × (L - 1) × L` XP. The level ceiling is 100, displayed as MAX LEVEL. Normal rank gates are 1/3/5/8/12. Switching ships restores that hull's existing XP, hardware and loadout without transferring another hull's XP or purchases.

The Scout begins at 100 world units/second with 260 units/second² acceleration. Ship XP adds 2.5% speed and acceleration per level after level 1, capped at level 12, independently of the purchased 4%-per-rank drive upgrade. Crafted ships start from 180 speed and 400 acceleration before their family bonuses. Sovereign retains its intrinsic high-performance drive (248.4 speed) without another XP multiplier.

Collected category points are retained immediately and survive death. Asteroid loot uses immutable birth provenance, so drifting a rock does not change its reward tier. Rocks born within [700,1400) world units are Haven belt: zero XP, an independent 25% basic category-point chance and 25% Tier 1-material chance. Other asteroid materials use a danger-derived birth tier 1..5. The hold carries at most 16 materials across five tiers; a full hold leaves uncollected material in space. Only physical home arrival banks it into matching stash tiers. Death, restart, return to title and schema 4 reload discard carried materials; banked stash and collected points remain. Crafting/upgrade recipes consume only banked tiers. Courier mission packets are separate from crafting materials and retain their existing delivery rules.

An expedition begins when the ship leaves Haven. Returning or dying records duration, peak distance, defeated enemies/asteroids/champions, actual point/XP/material gains and lost materials per tier. The most recent completed summary persists and is labelled LAST RECORDED EXPEDITION. A forced QA death before a new run may leave the previous summary intact; it is not relabelled as a newly invented expedition. Reloading preserves history without emitting a death event and exposes actual discarded cargo through transient material_loss/reason HUD fields.

## Save and diagnostic boundaries

Schema 4 retains campaign results, ship_xp[10] and prior exploration/boss counters, and adds shared abilities[17], per-hull loadouts[10] with hardware[6]/slots[3], five-tier cargo/storage and last-run materials_lost[5]. Blueprints must match real discovery/champion/victory state; crafting requires a blueprint; Sovereign ownership requires all patches, a complete league and first place in seeded standings; credits acknowledgement requires that victory. Slot IDs must identify an owned category-compatible item on an owned compatible hull.

Version 1..3 migration preserves recorded ordinary currency/XP/counters. Legacy scalar stored plus carried crafting materials convert once to Tier 1 stash, with cargo cleared. Old hardware moves only to the selected owned hull. Learned family effects remain crafted, and already crafted champion hulls retain their native ability ownership/loadout through the explicit compatibility migration. Legacy Sovereign receives royal defaults. Top-level ranks/unlocked remain derived compatibility mirrors, not a second global equipment authority. Schema 4 save validation/export preserves live carried cargo; starting a new Game after reload discards it rather than converting it to stash.

Live entities, current arena health and in-progress expedition timing are transient. Reloading returns to Haven. A league result is saved when a match finishes; an interrupted unfinished match remains unplayed. This is the solo checkpoint rule, not an authoritative competitive record.

Explicit `champion-0` through `champion-7` diagnostics create live fights for testing but never issue patches, blueprints, XP or match results. `tournament` moves to the beacon without granting qualification or wins. Production rewards always require the appropriate real encounter completion.

Historical native campaign/boss evidence covers sanctuary overrides, pixel hits, containment, live attacks, once-only rewards and losses/retries for its pinned source. Schema 4 acceptance additionally requires actual per-hull hardware/XP isolation, slot-gated normal/royal effects, atomic tiered recipes, belt birth-origin rewards, migration/reload loss and physical banking. The loadout view and shell-integration tests exercise actual TypeScript code with fake DOM/Game/save adapters; they do not prove native progression, IndexedDB, pixel painting or phone input. Preserve dated evidence identities, and record new normal-browser/save checks separately. Human balance, natural progression playthroughs, physical devices and sustained performance remain pending until actually measured.
