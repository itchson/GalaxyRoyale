# Galaxy Royale lore lexicon

The canonical game labels are **KIKAI**, **KESSHO** and **RAICHO** in ASCII Latin letters. These are Japanese-inspired words with authored meanings in Galaxy Royale. User canon governs the fiction; the references below establish the Japanese senses, not the game's history or mechanics. The machine-readable authoring source is `assets/narrative/lore-lexicon.json`.

| Game label and first introduction | Japanese reading | Lexical sense | Galaxy Royale use |
| --- | --- | --- | --- |
| KIKAI / technological species | 機械, きかい, kikai | Machine | The technological species descended from biological humanity. This species sense is fictional. |
| KESSHO / crystal | 結晶, けっしょう, kesshō | Crystal | Crystal horns and named crystal resources; existing resource tiers remain authoritative. |
| RAICHO / flight craft | 雷鳥, らいちょう, raichō | Rock ptarmigan; literal thunder bird | The fictional craft a KIKAI pilots. |

Shogakukan's *Digital Daijisen* gives the machine sense and reading for [機械](https://kotobank.jp/word/%E6%A9%9F%E6%A2%B0-49964). The National Diet Library records 結晶 with ケッショウ, the ASCII transliteration Kessho and the English equivalent Crystals in [Web NDL Authorities](https://id.ndl.go.jp/auth/ndlsh/00565650). KESSHO and RAICHO omit macrons in game labels; the long vowels remain visible in the source readings above.

The [Japan Tourism Agency's Rock Ptarmigans entry](https://www.mlit.go.jp/tagengo-db/en/R3-00356.html) identifies raichō as the real bird and explains its thunder-bird name. RAICHO is our fictional flight-craft usage, not the standard Japanese word for spacecraft. The game does not identify this craft with a real-world mythological Thunderbird tradition. Japanese bird inspiration and Galaxy Royale craft lore are separate claims.

## Starter display models

| Manufacturer | Canonical display model | Influence | Existing starter |
| --- | --- | --- | --- |
| PAN | PAN HIKARI | Light; exploration and passive support | Flux Drive |
| RENKO | RENKO HIBANA | Spark; offensive output | Pulse Lens |
| HUGO | HUGO MAMORI | Guarding/protection; defense | Guard Coil |

The meanings and readings are supported by Shogakukan's dictionary entries for [光 / hikari](https://kotobank.jp/word/%E5%85%89-119206), [火花 / hibana](https://kotobank.jp/word/%E7%81%AB%E8%8A%B1-612158) and [守り / mamori](https://kotobank.jp/word/%E5%AE%88%E3%82%8A-635824). MAMORI uses the guarding/defense sense; no divine protection mechanic is implied. These three model choices are authored names, not dictionary-defined craft models.

Introduce a model as `PAN HIKARI / RAICHO flight craft`, `RENKO HIBANA / RAICHO flight craft` or `HUGO MAMORI / RAICHO flight craft`. After that, use the model or RAICHO without repeating the explanation. Instructions can retain clear functional English: fly, aim, fire, hull, cargo, hangar and flight controls. Introduce KESSHO as a crystal before using the lore term alone.

## Identity and compatibility

Main manufacturer determines hair color, the starter's manufacturer presentation and the existing single starter ability. Sub-manufacturer determines crystal horn color and adds no extra ability or combat bonus. Arena sponsor is independent: sponsorship does not recolor hair/horns, change the license or grant an ability. This lexicon does not revise guide callsigns, character types, portraits or arena sponsorship lore.

This is a display-only naming change. Retain `pan`, `renko`, `hugo`, `dora`, `doro`, `talo`, `hex_id`, `selected_ship`, `ship_xp`, `loadouts` and the existing ability IDs. All three manufacturer variants retain starter hull **0** and its existing XP/loadout history. Do not hash translated labels into licenses or issue another starter grant. No save migration is required by these names.

Historical aliases are exactly **PAN Pathfinder**, **RENKO Lancer** and **HUGO Bulwark**, including their uppercase display variants. They are documentation/search aliases, not substring replacement rules. In particular, the separate champion craft **Bulwark** keeps its name. No general champion or later-hull rename is authorized by this lexicon; those names remain a future authored roster review.

## Validation boundary

Run `node --test scripts/test/lore-lexicon.test.mjs` for local authoring consistency. It checks canonical labels, functional English introductions, source references, scoped historical aliases, starter identity/ability mapping and the independent sponsor rule. It does not prove that runtime menus have adopted these labels, certify a translation, test a campaign playthrough or measure a physical phone. Runtime wording integration belongs to the gameplay and shell owners.

Sources were checked on **2026-10-11**. Japanese orthography is reference metadata; game display labels use the user's ASCII forms.
