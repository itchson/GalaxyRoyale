# Proposed wiki additions

For Root's canonical wiki integration. These paragraphs describe implemented source; they do not claim a new browser capture, completed natural training playthrough or physical performance test.

## Asteroids and materials

Asteroids have four material families: rock, metal, ice and porous. Their seeded outlines, facets, opaque crater floors and compact mineral deposits vary by instance. The fine occupied cells are the real collision body: dark crater shading is solid material, while damage can remove cells and create real holes. More distant tiers subtly grade the surface and tune size and motion. Material family and reward tier are fixed when an asteroid is born; knocking it into another area does not change its rewards.

The Haven training belt lies 700–1,400 world units from home. Its asteroids award **no XP**. A genuine defeat independently has a **25% chance of one material** and a **25% chance of one basic category point**. Haven materials are strictly T1. T1 porous asteroids can drop **Phase Silicate**, metal asteroids **Iron Filament**, and ice asteroids **Coolant KESSHO**; ordinary rock drops generic T1 material. These named items are labels within the T1 stock, not extra cargo capacity or a duplicate material. PAN training needs Phase Silicate, RENKO Iron Filament and HUGO Coolant KESSHO.

Remote site belts orbit outside their protected node: their birth annulus extends from the node radius plus240 to the node radius plus520. Their T1–T5 reward tier is resolved from that seeded site's distance; outer-field asteroids use their birth area's danger tier. Asteroids outside the Haven tutorial belt retain **2XP per genuine defeat**, awarded to the active RAICHO. Point pickups are retained; material cargo must physically return to home to bank. Death or expedition abandonment loses unbanked materials. Diagnostic scenes do not earn progression.

APR-1337's first practical task counts **10 genuine asteroid defeats of any family**. After that, collect **10 matching named items**, bank them and perform the first upgrade on the corporation's equipped factory ability. Dropping or losing cargo does not erase genuine collection history, but missing banked stock must be replaced before the recipe can be paid. Already-owned higher starter ranks are recognized without another charge or downgrade.

## Hull and shield chunks

A new factory RAICHO has **100 hull HP**, shown as **five 20HP hull chunks**. The marks represent actual maximum capacity and remaining health, including a partial final chunk when upgrades or bonuses give a non-multiple of20; they are not five separate collision objects. Reinforced Hull adds15 maximum HP per effective hardware rank.

**HUGO MAMORI starts with Guard Coil R1 equipped:30 shield HP in three 10HP shield chunks.** Guard Coil supplies that capacity; the licensed factory chassis has no additional innate shield. Fresh PAN HIKARI and RENKO HIBANA instead begin with their own passive/offensive ability and no shield until capacity is acquired. Anonymous legacy Scout and later hulls retain their existing30HP chassis shield policy. Unequipping Guard removes its capacity.

Guard Coil adds8 shield HP per later effective rank. R1 increases capacity but does not recharge; R2 unlocks recharge after four seconds without damage. Aegis Capacitors also grants10 capacity per effective hardware rank and unlocks recharge at R1. Effective ranks and the current RAICHO's level govern the rate. HUGO's active manufacturer affinity reduces incoming damage by10% only while a valid matching defensive ability is equipped; sub-manufacturer and sponsor do not grant that bonus.

## Evidence and remaining validation

The current asteroid implementation is `ebcaf46` after `59e6603`. Native core checks pass373 tests with one existing ignored export; the final contact comes from32 actual native packets. Source and native tests cover connected intact bodies, real damage holes, deterministic birth geometry, unchanged tutorial/remote reward rules and shield protection. Root's current native HUD source maps hull20HP/shield10HP capacities. Final integrated GPU/GL appearance, minimum-zoom legibility, frame cost and natural loot pacing remain separate validation work.
