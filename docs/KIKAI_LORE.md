# Kikai license and corporation canon

Kikai are the game's droid race. DORA is the female chassis designation with long hair, DORO male with short spiky hair, and TALO neutral with a crew cut. Player identities have a main manufacturer, a sub-manufacturer, and a cryptographically generated hexadecimal license hash. Hair shape follows the droid type; hair color follows the main corporation. Type-specific player portraits remain a separate art task; recoloring DORA does not produce a DORO or TALO portrait.

The full format is `TYPE-MAIN-SUB-HASH`. Lore shorthand takes the O/A/T type prefix, main initial and sub-manufacturer initial followed by `#HASH`. The guide is **DORA-RENKO-PAN-1337**, called **ORP#1337**. Female callsigns start with O, male with A, and neutral with T; the next initials identify main and then sub-manufacturer. Her red hair identifies RENKO Cybernetics; her green crystal horns identify PAN Dynamics as sub-manufacturer. The user explicitly confirmed RENKO spelling on 11 October 2026.

| Corporation | Main hair / hull / home color | Specialization | First ship |
|---|---|---|---|
| PAN Dynamics | Green | Exploration / passive | PAN Pathfinder |
| RENKO Cybernetics | Red | Weapons and technology / offensive | RENKO Lancer |
| HUGO CORP | Blue | Protection / defensive | HUGO Bulwark |

The sub-manufacturer independently colors crystal horns: PAN green, RENKO red, HUGO blue. Main manufacturer determines the first hull, its initial equipped ability, and Haven's visual theme. XP belongs to the individual ship; a new license starts with a level-one ship.

## Reference and runtime distinction

The user supplied DORA's original turnaround at `C:/Users/mitch/Desktop/xcnbxcnb.png`. The user then approved `assets/concepts/kikai/dora-reference-expression-sheet.png`, generated with the built-in OpenAI image tool from that reference, as the desired pixel detail benchmark. The original image is a reference, not a texture pasted into the game. The native portrait pipeline documents cropping, alpha threshold, resampling, palette reduction, manually authored animation layers, and their measured runtime output in `KIKAI_ART.md`.

Character portraits, ships, facilities and icons should retain authored pixel clusters through fractional object/layer transforms. Raster display resolution still determines final visible coverage; an arbitrary raster conversion is not a lossless promise.

## Initial mission and discoveries

ORP#1337 guides the license holder through actual flight, Haven-belt destruction, T1 material collection, physical home banking, and additional basic ability crafting/equipping. Persistent ordered stages derive from actual gameplay flags. Unfinished training remains available in the Journal.

Unknown ships and abilities are absent from catalog menus until actual world discovery, a blueprint or an unlock reveals them. Knowing a recipe, owning a crafted item and equipping that item are separate progression states. Existing saves retain their currency, ship XP, materials and loadouts; format migration does not issue a new license or starter grant.
