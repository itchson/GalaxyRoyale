# RAICHO shield and manufacturer affinity

The ordinary chassis has 30 shield HP, shown as three 10-HP native pixel-quad chunks. Shield capacity and charge are separate: an empty shield remains a visible capacity track. HUGO's factory Guard Coil rank 1 adds 20 HP, so that equipped starter legitimately shows five chunks.

Aegis Capacitors hardware adds 10 HP per effective rank. Guard Coil adds 20 HP at rank 1 and 8 HP for each additional effective rank. The effective ranks still obey the selected RAICHO's level gates. The Sovereign has an explicit additional 40-HP chassis capacity; it does not create an extra ability slot.

Factory shields do not recharge outside Haven. Aegis rank 1 or Guard Coil rank 2 installs recharge. After four seconds without taking damage, its rate is:

`3 HP/s × (1 + 0.2 × (rechargeRank − 1)) × (1 + 0.03 × (min(RAICHO level, 12) − 1))`

`rechargeRank = max(effective Aegis rank, effective Guard Coil rank − 1)`. Zero means locked. The Sovereign's existing enhanced hardware has a minimum 18 HP/s rate. Haven still repairs the hull, shield and destroyed body cells. The first real HUGO training upgrade, Guard Coil rank 1 to 2, installs recharge through the existing level-one training exception.

Each RAICHO has one offensive, one defensive and one passive ability slot. A slot may be empty. Owned compatible abilities can be swapped at Haven, and their displaced inventory remains owned. A manufacturer affinity applies only while a valid, owned, compatible ability is equipped in its matching slot:

| Main manufacturer | Slot | Current bonus |
| --- | --- | --- |
| PAN | Passive | 10% drive speed and acceleration |
| RENKO | Offensive | 10% derived weapon damage |
| HUGO | Defensive | 10% incoming damage reduction |

The starter uses the license's main manufacturer. Collectible RAICHO use their authored manufacturer independently of the pilot, sponsor and crystal-horn sub-manufacturer. The Sovereign currently has no separately authored manufacturer and retains the license's main affinity. This rule does not rewrite the KIKAI identity or hash.

The curved hull track keeps its eleven marks. Shield extraction uses actual maximum capacity, including a fractional final chunk. Above eleven chunks, the complete horizontal field and spacing compress together inside the existing 44 × 12 logical-unit envelope. The valid maximum is 172 HP (18 chunks). Capacity changes reset presentation trails rather than inventing damage; fractional quad transforms preserve detail. Native geometry tests check actual quad separation and fill clipping, rather than a screenshot approximation.

Schema 6 persists the existing ranks, loadouts, inventory and RAICHO XP. No new shield balance fields are saved. Reload derives current capacity and recharge from those records. Automated fixtures establish these rules; natural progression balance and physical-device performance require separate playtests.
