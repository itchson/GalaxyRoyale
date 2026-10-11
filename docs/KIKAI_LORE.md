# KIKAI, MEISHI CARD and corporation canon

KIKAI are technological life descended from humans in a future where biological life can no longer survive. They are people with friendships, ambitions and hobbies. DORA is the female chassis designation with long hair, DORO male with short spiky hair, and TALO neutral with a crew cut. Player identities have a main manufacturer, a sub-manufacturer, and a cryptographically generated hexadecimal identity hash. Hair shape follows the droid type; hair color follows the main corporation. DORA, DORO and TALO each have an authored native portrait pack. The creator and **MEISHI CARD** share the selected type, main-hair color and sub-manufacturer horn color; recoloring DORA does not stand in for the other silhouettes.

The setting calls its flight craft **RAICHO**, inspired by the Japanese bird name, and crystals **KESSHO**. The exact ASCII spellings, sourced meanings and starter model names are recorded in [the lore lexicon](LORE_LEXICON.md). These display terms do not rename save keys, redistribute XP or grant another starter.

Anatomy is authored separately for each type. DORA has a feminine chest; DORO has a masculine, flat mechanical chest with angular plate construction; TALO has a neutral, flat torso. DORO and TALO armor must not inherit rounded breast forms or cup-shaped seams from the DORA reference. Hair length and recoloring alone do not establish the type's silhouette. Inspect the source art and the actual native portrait at its intended display size.

The full format is `TYPE-MAIN-SUB-HASH`. Compact names use **A for female, O for male and T for neutral**, followed by main and sub-manufacturer initials. The guide is **DORA-PAN-RENKO-1337**, called **APR-1337**. Her green hair identifies PAN Dynamics; her red crystal horns identify RENKO Cybernetics as sub-manufacturer. The user's latest clarification on 11 October 2026 supersedes the earlier reversed A/O prefixes and OPR/ORP guide names. RENKO retains its confirmed spelling. The player designation formatter retains the previously specified `#HASH`; APR-1337 is the guide's authored display name. Neither that prefix correction nor the MEISHI CARD display name rewrites a saved identity, hash or progression record.

| Corporation | Main hair / hull / home color | Specialization | First RAICHO |
|---|---|---|---|
| PAN Dynamics | Green | Exploration / passive | PAN HIKARI |
| RENKO Cybernetics | Red | Weapons and technology / offensive | RENKO HIBANA |
| HUGO CORP | Blue | Protection / defensive | HUGO MAMORI |

The sub-manufacturer independently colors crystal horns: PAN green, RENKO red, HUGO blue. Main manufacturer determines the first hull, its initial equipped ability, and Haven's visual theme. XP belongs to the individual RAICHO; a newly registered KIKAI starts with a level-one RAICHO.

## MEISHI CARD and personal collections

The active player-facing identity record is **MEISHI CARD**. It reuses the selected identity's animated native profile portrait (portrait ID 9), designation and manufacturers. APR-1337 remains the separate authored guide portrait (ID 8). The card reads the authoritative current RAICHO, its own level/XP, hull/shield, three equipped ability categories and retained RENKO offensive, HUGO defensive and PAN passive point balances. It is a view of existing progression, not a second inventory or a new character grant.

There are exactly **eight arena patch slots**, one for each stable champion index. Only an actually earned patch occupies its slot; empty slots do not invent an opponent's identity or reward. Repainting the card preserves its nodes and scroll/focus while values change. The MEISHI CARD and collection routes are source-integrated work whose current browser presentation remains **verification pending** until evidence names the tested normal build.

Personal collections have distinct meanings:

| View | What it records |
|---|---|
| Cargo inventory | Carried versus banked materials, plus separate mission-item state. Phase Silicate, Iron Filament and Coolant KESSHO are named subsets of T1 stock, not additional capacity or duplicate material units. |
| Hull collection | Revealed hull/blueprint discoveries and separately crafted or rewarded ownership. Each owned RAICHO retains its own XP, hardware and loadout; selection does not transfer XP between hulls. |
| Ability collection | Discovered blueprints, shared crafted ownership/rank and the active RAICHO's equipped offense, defense and passive slots. Discovery is not ownership; crafting does not auto-equip; replacing a slot retains the displaced owned item. |

Material cargo banks only on physical home arrival and remains at risk until then. Courier packets are mission state, retained separately from the material capacity; a Haven delivery still requires reporting to its source beacon to claim. Current recipes spend raw material tiers and the explicit named-material training recipe. A separate crafted-component inventory and component-to-item recipes are **planned**, not part of today's save or crafting system. Unknown hulls and abilities must remain hidden until an actual discovery, blueprint or unlock reveals them; current collection filtering and layout have their own integration checks.

Existing source/save identifiers containing `license`, including `factory_license` and `kikai_license`, remain compatibility and implementation names. Historical evidence and software asset-license/provenance records retain their original meanings. Changing the visible identity label to MEISHI CARD does not rename that ABI or alter software licensing.

## Factory starter equipment

A fresh registration equips one owned rank-one ability on its level-one factory RAICHO. The licensed factory chassis has no innate shield: defense comes from an equipped ability or purchased shield hardware. Anonymous legacy Scout and later chassis retain their innate 30 shield HP. A sub-manufacturer changes horn color; it does not add another factory item or a combat bonus.

| Factory RAICHO | Equipped starter | Actual fresh effect |
|---|---|---|
| PAN HIKARI | Flux Drive / passive | +4% speed and acceleration, multiplied by the active PAN +10% drive affinity: top speed 114.4 world units/s and acceleration 297.44 world units/s². |
| RENKO HIBANA | Pulse Lens / offense | +10% primary damage, multiplied by the active RENKO +10% weapon affinity: primary damage 14.52. Every fifth actual primary attempts an extra bolt at 150% of current primary damage, initially 21.78. |
| HUGO MAMORI | Guard Coil / defense | 30 shield HP in three 10-HP chunks, plus the active HUGO 10% incoming damage reduction. Rank one does not recharge; effective rank two grants 38 HP and unlocks recharge after four seconds without damage. |

The PAN figures are derived movement limits, not a claim that the RAICHO is already moving. The RENKO bolt travels at 500 world units/s plus 25% of the RAICHO velocity. Only admitted primary shots advance its counter. Projectile capacity can refuse a primary or the extra bolt; a refused fifth-shot bolt is discarded and never replayed. Pulse Echo is a separate boss ability, not the factory fifth-shot effect.

Item ownership, equipment and manufacturer affinity remain distinct. Unequipping the starter removes its item effects; an empty or invalid matching slot disables the hull's affinity. Another owned compatible item in the matching category can activate that affinity. The active hull's manufacturer governs the bonus independently of pilot, secondary manufacturer and sponsor. These rules derive from existing loadout and XP records; they do not add a license, grant points or reset progression.

The authoritative effect registry, shot admission and shield rules are in `crates/game_core/src/abilities.rs`, `lib.rs` and `shield_progression.rs`. Native rule checks are separate from natural progression, subjective balance, browser cue readability and physical-device acceptance.


## APR-1337, the player's friend

APR-1337 is extremely old despite her youthful feminine appearance. She existed when biological humans still lived and now devotes herself to keeping humanity alive through the KIKAI. Newly created players are new KIKAI entering the Galaxy Royale universe; she welcomes and helps them because that continuity matters to her. No exact age, human-era date or mind-transfer mechanism is invented.

She is a friendly explorer with dreams beyond her flight-guide duties and performs as a K-pop star as her hobby. Her long life informs her care without making her humor, ambitions or hobbies disappear. Dialogue should sound like a capable friend sharing an adventure: warm, curious and clear, without replacing the player's choices. Her music hobby is character canon; songs, performances and voice assets require their own authored and reviewed production work.

## Galaxy Royale and sponsorship

Galaxy Royale is the universe's largest and most popular sport. Arena champions compete for recognition, and manufacturer corporations sponsor arenas and competitors. A sponsor is distinct from a KIKAI's main and sub-manufacturers: sponsorship does not recolor hair or horns, change an identity, or grant a RAICHO ability.

The current eight encounters have authored KIKAI identities and independent sponsors, recorded in the campaign table and DevBible. The user's example **APP#F344FEE** is a female DORA-PAN-PAN competitor representing PAN in a PAN-sponsored arena; **ORP#2323** is a male DORO-RENKO-PAN competitor also sponsored by PAN. Historical character/model aliases and stable encounter indices are retained. Arena presentation identifies sponsor, champion identity, patch and blueprint separately. Discovery gates still hide unknown opponents and rewards. Additional roster characters remain future authored content.

## Reference and runtime distinction

The user supplied DORA's original turnaround at `C:/Users/mitch/Desktop/xcnbxcnb.png`. The user then approved `assets/concepts/kikai/dora-reference-expression-sheet.png`, generated with the built-in OpenAI image tool from that reference, as the desired pixel detail benchmark. The original image is a reference, not a texture pasted into the game. The native portrait pipeline documents cropping, alpha threshold, resampling, palette reduction, manually authored animation layers, and their measured runtime output in `KIKAI_ART.md`.

Character portraits, ships, facilities and icons should retain authored pixel clusters through fractional object/layer transforms. Raster display resolution still determines final visible coverage; an arbitrary raster conversion is not a lossless promise.

## Initial mission and discoveries

APR-1337's first practical training mission is to destroy **10 asteroids of any type**. The next mission is to collect **10 specific materials from designated asteroid varieties**, bank them at home, then perform the first upgrade on the main corporation's factory-equipped ability: PAN passive Flux Drive, RENKO offensive Pulse Lens, or HUGO defensive Guard Coil. The factory item starts at rank 1; its first training upgrade is rank 2, available to the level-one starter RAICHO using the actual named-material recipe. This replaces the earlier one-rock/additional-ability tutorial. Counted training and migration exist in source; natural loot pacing and a complete player-earned playthrough remain separately pending.

The Journal must show the current count, target asteroid and material, matching slot and live upgrade recipe. Progress comes from real destructions, collected items, physical home banking and the successful upgrade. Merely damaging a rock or entering a diagnostic scene cannot advance the mission. Carried crafting items remain at risk on death; banked materials and collected points remain at home. Unfinished training remains available in the Journal, and a completed earlier tutorial does not revoke earned progression.

Unknown RAICHO and abilities are absent from catalog menus until actual world discovery, a blueprint or an unlock reveals them. Knowing a recipe, owning a crafted item and equipping that item are separate progression states. Existing saves retain their currency, per-RAICHO XP, materials and loadouts; format migration does not issue a new license or starter grant.

Playable co-op remains pending. The generated PAN co-op dock reference and its converted native128/native256 candidates are review material, not the shipping dock or proof of a playable shared universe.
