# Galaxy Royale DevBible

The DevBible is the versioned, source-backed reference for the game. Open **DevBible** in the private local Mission Control sidebar. It supports text search, category/status filters and links such as `#wiki/champion-0`.

The first catalog contains 33 entries: the eight named champions, ten-hull fleet identity, seventeen automatic abilities, six biomes, canonical seed nodes, Haven and remote belts, courier/bounty missions, ship XP/material rules, ordinary families and fifteen bosses, Riveter, Radiation Mote, pixel-engine/art contracts, verification boundaries and future modes.

## Meaning of status

- **Implemented** describes actual source behavior. It does not assert balance, phone performance or complete natural progression acceptance.
- **Verification pending** names remaining acceptance work.
- **Planned** explicitly excludes the feature from current runtime claims.

Player-facing prose uses **missions**. Existing save and Rust fields named `quests` remain unchanged. Do not infer a migration from the terminology change.

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
