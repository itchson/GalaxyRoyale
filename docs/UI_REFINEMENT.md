# Quiet contextual pixel interface

## Direction and references

This is an original Galaxy Royale interface. The references inform information hierarchy and interaction; their fonts, icon art and layouts are not imported.

[Halo Infinite's UX/UI developer interview](https://www.halowaypoint.com/news/inside-infinite-october-2021) explains reducing persistent framing and unnecessary corner scanning, allowing the world to lead, and communicating important state through contrast, motion and clean messaging. It also describes pairing color with recognizable shape. Our interpretation is a quiet flight surface, a few stable navigation icons, a clear map and contextual detail rather than continuously displaying every statistic.

[Overwatch's official May 2026 notes](https://overwatch.blizzard.com/en-gb/news/patch-notes/live/2026/05/) describe ability-specific timing information in the Advanced Info Panel. Our interpretation is to retain exact mechanics in an explicit Details view while keeping the live RAICHO overview concise. This source supports contextual mechanics information; it is not a claim that Blizzard prescribes our spacing or typography.

The [Stadium design presentation](https://media.gdcvault.com/gdc2026/Slides/Hwang_Scott_DesigningStadium.pdf) is an additional primary reference for the relationship between combat information and ability cards. Only retrieved text was available in this research pass; no full visual-slide review is claimed.

## Original hierarchy

- Flight: small authored navigation icons, top-right map and right-aligned telemetry. Hover/focus reveals the facility or action context. Critical hull/shield state stays near the RAICHO.
- MEISHI CARD: a standalone read-only card with the selected animated native profile and callsign, current RAICHO with its own level/XP and equipped slots, retained corporate points, and exactly eight arena patch slots. Close is its only control. No cargo/hull/ability/stats links, tabs or buttons, Settings action, or save/update footer appears in the card. Readouts are not navigation. A dedicated native `meishi-panel` and all card content share one local UI group; repaint preserves focused nodes.
- Live RAICHO menu: Escape and the status toolbar open current hull/shield, one ability per category and carried-cargo risk while the universe continues. Its MEISHI entry opens the separate card. Cargo inventory, hull collection and ability collection are accessible here as dedicated views; optional technical details stay here rather than adding controls to the card. Materials and mission packets remain separate, as do discovered, owned and equipped items.
- Settings: one heading per group, concise names, measured controls and quiet row separators. Hover/focus changes appearance without moving targets. Touch controls retain at least 44 CSS pixels of interaction area. Desktop inspection panels have physical width caps independent of world zoom.
- KIKAI creation: type, main manufacturer/hair, and sub-manufacturer/crystal horns remain separate. The live portrait conveys appearance and is reused on the MEISHI CARD; the short starter summary names the RAICHO and its ability rather than repeating every appearance token. Existing source/save identifiers containing `license` are unchanged.

RENKO offensive, HUGO defensive and PAN passive balances and slots retain that order across the card and collections. A revealed recipe, a crafted item and an equipped item need distinct concise labels. Today's crafting uses raw tiers and the named-material training recipe; component crafting is planned and must not appear as owned cargo or an enabled recipe. Save state is reported independently from the identity card.

## Collectible MEISHI card direction

The next card refinement is a collectible native object, rather than another menu frame. Its overall theme follows the pilot's **main manufacturer**: PAN green, RENKO red or HUGO blue. Secondary crystal-horn color remains a separate portrait role; it does not recolor the card or change the retained balances. A stepped graphite/platinum chassis, corporate enamel accents, readable identity and existing portrait animation form the hierarchy. The card retains the selected profile's original fine source cells.

The actual generated PAN reference is `artifacts/concepts/meishi-collectible-20261011/pan-collectible-reference.png`, with its prompt retained beside it. It supplies art direction for material depth, profile/RAICHO composition and eight recessed chamfered/octagonal sockets. Its microtext, slogans, numeric balances, portrait and craft drawing are not new canon or runtime assets. Native cached parts are separately authored to adapt that reference; no automatic pixel-perfect raster conversion is claimed.

Exactly eight patch sockets remain visible. Every unearned socket has an opaque dark interior, with no competitor portrait, emblem, name or sponsor revealed beneath it. Only an actually earned patch exposes its identity. Empty sockets are a visible collection structure, not disabled rewards or actions. Close remains the only control: no tabs, collection links, Settings, save/update footer or new gameplay actions belong inside the card.

The proposed normal layout is a portrait card with a clear identity/portrait area and a separate eight-socket collection area. Short landscape windows use a compact arrangement of the same content, without adding a second portrait or replacing the selected identity. The measured Close target and card content stay inside the viewport; any required scrolling remains within the dedicated card. Exact frame geometry and responsive acceptance await the native implementation and actual normal-build captures.

Composition must fit the existing 60,000 submitted-cell and 120,000 work limits. Clipped or alpha-zero occupied portrait cells still cost budget; fractional display fitting does not reduce that cost. A quiet stretched fill plus sparse cached bevel, etched-metal and socket parts is preferable to a densely occupied full backing grid. The existing conservative card fixture reaches 54,023 cells, including controls removed from the standalone card. New decorative art therefore needs a measured dedicated Close-only composition, not an assumption that its individual objects fit. Whole-element admission must preserve all portrait layers and every earned patch; tests should compare the actual emitted components as well as the total.

This section records the proposed design and composition constraints. The earlier standalone-card browser receipts remain evidence of that earlier presentation only; they do not accept the new collectible art.

## Typography and motion

Body text uses the engine's RoyaleMono occupied-cell glyphs. Shell font sizes describe semantic layout; native glyph size and clipping must fit that measured area. Shrinking CSS while rounding the requested native scale upward is insufficient. Fractional cell transforms are appropriate when the native pixel renderer can fit the complete glyph and line inside the measured region.

Default shell text scales from 8 logical units within 12–16 CSS pixels; captions from 7 within 11–14; headings from 12 within 18–24. Physical upper bounds keep wide-screen menus compact. Interaction targets retain 44–52 CSS pixels independently of text size; Settings row padding and license status portraits also have physical upper bounds. These are authored starting values, not a universal readability guarantee. Native map/footer microtext is not substituted for menu body text.

Existing authored 64-cell control masks retain neutral metal and category accents. Hover, focus, pressed and save states provide bounded feedback; decorative motion must respect Reduced Motion. No continuous border animation or moving hit regions are needed to explain an action.

## Verification boundaries

Source tests must verify truthful state, one-slot order, exactly eight patch slots, card-only Close, hidden card navigation/Settings/save footer, live RAICHO-menu entry routes, disclosure persistence, missing-affinity clearing, keyboard controls and native glyph containment. Actual settled browser captures must verify compact and larger layouts with both GPU backends. `artifacts/meishi-20261011/README.md` records the earlier standalone card's bounded desktop WebGPU/WebGL2 and keyboard/scroll observations. New collectible art requires its own source, composition and tested-build receipts; the earlier images do not verify it. Human readability, physical-device controls and sustained performance remain separate gates. Evidence records the tested source/build identity and distinguishes source art, candidate conversion and shipping runtime art. A co-op dock contact sheet does not establish playable co-op.
