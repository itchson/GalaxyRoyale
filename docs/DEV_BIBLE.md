# Galaxy Royale DevBible

The DevBible is the versioned, source-backed reference for the game. Open **DevBible** in the private local Mission Control sidebar. It supports text search, category/status filters and links such as `#wiki/champion-0`.

The original catalog contained 33 entries; the current catalog also records KIKAI manufacturers, APR-1337 and the real first-flight mission. KIKAI are technological descendants of humans. The guide's friendship, exploration and K-pop hobby, Galaxy Royale's sporting role and independent corporation sponsorship are accepted lore. The current eight champions have authored KIKAI identities; additional roster content remains planned. Coverage includes ten-hull fleet identity, seventeen automatic abilities, six biomes, canonical seed nodes, Haven and remote belts, courier/bounty missions, per-RAICHO XP/material rules, ordinary families and fifteen bosses, Riveter, Radiation Mote, pixel-engine/art contracts, verification boundaries and future modes.

## Meaning of status

- **Implemented** describes actual source behavior. It does not assert balance, phone performance or complete natural progression acceptance.
- **Verification pending** names remaining acceptance work.
- **Planned** explicitly excludes the feature from current runtime claims.

Player-facing prose uses **missions**. Existing save and Rust fields named `quests` remain unchanged. Do not infer a migration from the terminology change.

The lore lexicon uses **KIKAI** for the technological species, **KESSHO** for crystals and **RAICHO** for flight craft. Current starter display models are **PAN HIKARI**, **RENKO HIBANA** and **HUGO MAMORI**. `#wiki/lore-lexicon` links the source readings, naming policy and stable save identifiers. Native schema6 counted training and independent type/main/sub profile portraits are integrated; their source, browser and natural-play verification remain separate.

The active identity display is **MEISHI CARD**: a standalone read-only card containing the selected animated native profile, current RAICHO with its own XP and equipped slots, retained corporate points, and exactly eight arena patch slots. **Close is its only control**: no collection/stats navigation, tabs, Settings or save/update footer appears inside the card. RENKO is offensive, HUGO defensive and PAN passive; the card does not change those mechanics or issue another identity.

The collectible-card frame is **implemented** as original cached native fittings, adapted from the generated PAN concept whose `artifacts/concepts/meishi-collectible-20261011/provenance.json` records its reference-only use. Graphite/platinum parts, corporate decks and enamel rails follow green PAN, red RENKO or blue HUGO main manufacturer while preserving the independently colored profile horns. Exactly eight opaque dark sockets reveal only earned patches. A portrait card and compact landscape arrangement retain the selected animated profile, read-only content and Close-only interaction. Continuous fitting preserves fine portrait/patch cells; the actual nine-combination all-earned native fixture reaches at most **45,367/60,000** submitted cells without raising the shared cap. A quota-rejected socket also suppresses its paired emblem.

Root's bounded normal-build desktop browser observations cover PAN responsive/keyboard/scroll presentation, actual WebGL2 diagnostics and the RENKO/HUGO themes. `artifacts/meishi-collectible-20261011/README.md` retains their tested builds and scope; these are Root's observations, not an additional browser run by this documentation author. Physical-phone controls, all-eight-earned browser progression, human readability and a 30-minute performance/memory soak remain **verification pending**. Earlier standalone-card evidence retains its original scope. `docs/UI_REFINEMENT.md` describes the current native composition and responsive contract.

Escape and the status toolbar open the live RAICHO menu while the universe runs. A MEISHI entry opens the separate card; Cargo, Hull and Ability collections are accessed through the RAICHO menu, not through the card. Cargo inventory separates material stock from mission packets. Hull collection distinguishes discovery from crafted ownership; ability collection distinguishes discovered, owned and equipped state. The earlier standalone card has scoped desktop WebGPU/WebGL2 and keyboard/scroll receipts in `artifacts/meishi-20261011/README.md`. Those receipts do not verify the new collectible frame, a physical phone or full natural progression.

Current crafting spends raw material tiers, with named T1 stock used by the explicit training recipe. Named stock is a subset of T1, not a second copy. A separate crafted-component inventory and component recipes are planned. Playable co-op is also pending; the PAN co-op dock concept and native conversion candidates are not shipping assets or gameplay acceptance. Internal/save names containing `license` remain stable, and software asset-license records retain their legal/provenance meaning.

## Sources and images

`docs/wiki/catalog.json` is structured data, schema version 1. Entries contain facts, prose sections, related-entry IDs, exact local references and classified media. No raw HTML or executable content is rendered. The reader escapes text and constructs only local source/media route links.

Authoritative behavior lives in Rust simulation and validated shell code. Source references open as plain text. Named table and contract regressions compare the catalog with the actual champion table, ability registry, biome weights, boss names, rank gates, schema, cell occupancy and pixel-packing constants. They deliberately do not hardcode a release/build identity: current evidence belongs to Mission Control's actual records.

Images are tracked contact sheets and bounded actual runtime captures. Authored runtime-art previews, converted generated-concept atlas previews and actual runtime screenshots are different classifications. Provenance links accompany every image. PNG preview files are inspection material; runtime sprites use palette-indexed occupied cell atlases.

The presentation gallery retains three exact images from source commit `cac1f03d2cdaab1059afad91e45aa591f82996bc`: a software contact sheet of actual extracted native cells and two observed 1280×720 WebGPU captures (Haven and a QA-console-admitted asteroid belt). The original `webgpu-haven.png` and `webgpu-belt.png` contain JPEG bytes; the Wiki copies use `.jpg` names and `image/jpeg` without changing those bytes. The actual native contact sheet remains PNG. Their tracked `EVIDENCE_SCOPE.md`, `ART_REVIEW.md` and `HUD_HANDOFF.md` describe the historical source/cache and acceptance limits. They do not demonstrate natural belt travel, mobile performance, later-build visuals or human audio comfort. Copying these six image/Markdown files did not import or edit that commit's production ledger.

## Local API

`GET /api/wiki` returns the catalog. Optional `q`, `category` and `status` filters are bounded and validated. `GET /api/wiki/:id` returns an entry. `GET /api/wiki/source?path=...` and `/api/wiki/media?path=...` only read exact catalog-listed canonical workspace files under docs, crates, apps, assets or artifacts. Source text has `nosniff` and a sandbox CSP. Media permits verified PNG/JPEG/WebP headers; SVG/HTML/audio are excluded. Symlink/junction ancestors, unlisted files, traversal, hidden files and arbitrary external URLs fail closed. This is not an arbitrary filesystem or secret-reading endpoint.

## Updating facts

Edit the catalog and reviewed prose on a feature branch when implementation changes. Include source references and provenance; keep planned content distinct. Run `node --experimental-transform-types --test apps/studio/test/*.test.mjs` and `pnpm exec tsc --noEmit`. Studio remains excluded from the private game package. Browser layout/accessibility and real device coverage require actual observation after integration.

Project phase metadata has a separate revision-checked API. `PATCH /api/project` accepts `{author,source,expectedRevision,patch:{phase,phaseDescription}}`. It permits those two metadata fields only, retains prior documents/activity and rejects stale snapshots. Use the Studio phase editor or `node scripts/studio-cli.mjs project update --file <UTF-8 JSON>`. Never hand-edit a live production store. The DevBible feature does not itself rewrite the project's recorded phase or task states.
