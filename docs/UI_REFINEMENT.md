# Quiet contextual pixel interface

## Direction and references

This is an original Galaxy Royale interface. The references inform information hierarchy and interaction; their fonts, icon art and layouts are not imported.

[Halo Infinite's UX/UI developer interview](https://www.halowaypoint.com/news/inside-infinite-october-2021) explains reducing persistent framing and unnecessary corner scanning, allowing the world to lead, and communicating important state through contrast, motion and clean messaging. It also describes pairing color with recognizable shape. Our interpretation is a quiet flight surface, a few stable navigation icons, a clear map and contextual detail rather than continuously displaying every statistic.

[Overwatch's official May 2026 notes](https://overwatch.blizzard.com/en-gb/news/patch-notes/live/2026/05/) describe ability-specific timing information in the Advanced Info Panel. Our interpretation is to retain exact mechanics in an explicit Details view while keeping the live RAICHO overview concise. This source supports contextual mechanics information; it is not a claim that Blizzard prescribes our spacing or typography.

The [Stadium design presentation](https://media.gdcvault.com/gdc2026/Slides/Hwang_Scott_DesigningStadium.pdf) is an additional primary reference for the relationship between combat information and ability cards. Only retrieved text was available in this research pass; no full visual-slide review is claimed.

## Original hierarchy

- Flight: small authored navigation icons, top-right map and right-aligned telemetry. Hover/focus reveals the facility or action context. Critical hull/shield state stays near the RAICHO.
- RAICHO overview: compact license portrait and callsign; level/XP; hull/shield; one ability per category; carried cargo and retained points. Technical output and dock inventory are available through an explicit button without rebuilding focused controls on each simulation tick.
- Settings: one heading per group, concise names, measured controls and quiet row separators. Hover/focus changes appearance without moving targets. Touch controls retain at least44 CSS pixels of interaction area.
- License creation: type, main manufacturer/hair, and sub-manufacturer/crystal horns remain separate. The live portrait conveys appearance; the short starter summary names the RAICHO and its ability rather than repeating every appearance token.

## Typography and motion

Body text uses the engine's RoyaleMono occupied-cell glyphs. Shell font sizes describe semantic layout; native glyph size and clipping must fit that measured area. Shrinking CSS while rounding the requested native scale upward is insufficient. Fractional cell transforms are appropriate when the native pixel renderer can fit the complete glyph and line inside the measured region.

Default shell text is12 CSS pixels minimum /8 logical units; captions11 /7; headings18 /12. These are authored starting values, not a universal readability guarantee. Small target sizes are independent from these text sizes. Native map/footer microtext is not substituted for menu body text.

Existing authored64-cell control masks retain neutral metal and category accents. Hover, focus, pressed and save states provide bounded feedback; decorative motion must respect Reduced Motion. No continuous border animation or moving hit regions are needed to explain an action.

## Verification boundaries

Source tests must verify truthful state, one-slot order, disclosure persistence, missing-affinity clearing, keyboard controls and native glyph containment. Actual settled browser captures must verify compact and larger layouts with both GPU backends. Human readability, physical-device controls and sustained performance remain separate gates. Evidence records the tested source/build identity and distinguishes source art from runtime art.
