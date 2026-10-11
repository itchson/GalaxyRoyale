# Kikai license and corporation canon

KIKAI are technological life descended from humans in a future where biological life can no longer survive. They are people with friendships, ambitions and hobbies. DORA is the female chassis designation with long hair, DORO male with short spiky hair, and TALO neutral with a crew cut. Player identities have a main manufacturer, a sub-manufacturer, and a cryptographically generated hexadecimal license hash. Hair shape follows the droid type; hair color follows the main corporation. DORA, DORO and TALO each have an authored native portrait pack. The creator and issued license share the selected type, main-hair color and sub-manufacturer horn color; recoloring DORA does not stand in for the other silhouettes.

The setting calls its flight craft **RAICHO**, inspired by the Japanese bird name, and crystals **KESSHO**. The exact ASCII spellings, sourced meanings and starter model names are recorded in [the lore lexicon](LORE_LEXICON.md). These display terms do not rename save keys, redistribute XP or grant another starter.

Anatomy is authored separately for each type. DORA has a feminine chest; DORO has a masculine, flat mechanical chest with angular plate construction; TALO has a neutral, flat torso. DORO and TALO armor must not inherit rounded breast forms or cup-shaped seams from the DORA reference. Hair length and recoloring alone do not establish the type's silhouette. Inspect the source art and the actual native portrait at its intended display size.

The full format is `TYPE-MAIN-SUB-HASH`. Compact names use **A for female, O for male and T for neutral**, followed by main and sub-manufacturer initials. The guide is **DORA-PAN-RENKO-1337**, called **APR-1337**. Her green hair identifies PAN Dynamics; her red crystal horns identify RENKO Cybernetics as sub-manufacturer. The user's latest clarification on 11 October 2026 supersedes the earlier reversed A/O prefixes and OPR/ORP guide names. RENKO retains its confirmed spelling. The current player license formatter retains the previously specified `#HASH`; APR-1337 is the guide's authored display name. No saved identity, hash or progression is rewritten by the display-prefix correction.

| Corporation | Main hair / hull / home color | Specialization | First RAICHO |
|---|---|---|---|
| PAN Dynamics | Green | Exploration / passive | PAN HIKARI |
| RENKO Cybernetics | Red | Weapons and technology / offensive | RENKO HIBANA |
| HUGO CORP | Blue | Protection / defensive | HUGO MAMORI |

The sub-manufacturer independently colors crystal horns: PAN green, RENKO red, HUGO blue. Main manufacturer determines the first hull, its initial equipped ability, and Haven's visual theme. XP belongs to the individual RAICHO; a new license starts with a level-one RAICHO.

## Factory starter equipment

A fresh license equips one owned rank-one ability on its level-one factory RAICHO. The licensed factory chassis has no innate shield: defense comes from an equipped ability or purchased shield hardware. Anonymous legacy Scout and later chassis retain their innate 30 shield HP. A sub-manufacturer changes horn color; it does not add another factory item or a combat bonus.

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

Galaxy Royale is the universe's largest and most popular sport. Arena champions compete for recognition, and manufacturer corporations sponsor arenas and competitors. A sponsor is distinct from a KIKAI's main and sub-manufacturers: sponsorship does not recolor hair or horns, change a license, or grant a ship ability.

With the corrected A/O prefixes, the user's example **APP#F344FEE** is a female DORA-PAN-PAN candidate representing PAN in a PAN-sponsored arena. **ORP#2323** is a male DORO-RENKO-PAN candidate who could also be sponsored by PAN. These are authored roster candidates; the existing eight encounters must not silently be renamed or have their progression reset. Arena presentation should identify sponsor, champion identity, patch and blueprint separately. Discovery gates still hide unknown opponents and rewards.

## Reference and runtime distinction

The user supplied DORA's original turnaround at `C:/Users/mitch/Desktop/xcnbxcnb.png`. The user then approved `assets/concepts/kikai/dora-reference-expression-sheet.png`, generated with the built-in OpenAI image tool from that reference, as the desired pixel detail benchmark. The original image is a reference, not a texture pasted into the game. The native portrait pipeline documents cropping, alpha threshold, resampling, palette reduction, manually authored animation layers, and their measured runtime output in `KIKAI_ART.md`.

Character portraits, ships, facilities and icons should retain authored pixel clusters through fractional object/layer transforms. Raster display resolution still determines final visible coverage; an arbitrary raster conversion is not a lossless promise.

## Initial mission and discoveries

APR-1337's first practical training mission is to destroy **10 asteroids of any type**. The next mission is to collect **10 specific materials from designated asteroid varieties**, bank them at home, then perform the first upgrade on the main corporation's factory-equipped ability: PAN passive Flux Drive, RENKO offensive Pulse Lens, or HUGO defensive Guard Coil. The factory item starts at rank 1; its first upgrade is rank 2, available to the level-one starter ship. This replaces the earlier one-rock/additional-ability tutorial; implementation and migration are tracked on the board until validated.

The Journal must show the current count, target asteroid and material, matching slot and live upgrade recipe. Progress comes from real destructions, collected items, physical home banking and the successful upgrade. Merely damaging a rock or entering a diagnostic scene cannot advance the mission. Carried crafting items remain at risk on death; banked materials and collected points remain at home. Unfinished training remains available in the Journal, and a completed earlier tutorial does not revoke earned progression.

Unknown ships and abilities are absent from catalog menus until actual world discovery, a blueprint or an unlock reveals them. Knowing a recipe, owning a crafted item and equipping that item are separate progression states. Existing saves retain their currency, ship XP, materials and loadouts; format migration does not issue a new license or starter grant.
