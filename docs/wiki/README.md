# Catalog maintenance

The first-class Studio DevBible reads `catalog.json`, schema version 1. The JSON is intentionally typed content rather than Markdown HTML. Its seven categories group project/verification, champions, fleet/equipment, enemies, universe, missions/progression and engine/art.

Each entry has a stable slug, title, category, implementation status, summary, tags, fact pairs, prose sections, related slugs, references and classified media. Optional `data` stores bounded machine-readable source facts for drift tests; it grants no gameplay authority. `implemented`, `planned`, and `verification-pending` are the only statuses. Reference roles describe why the source matters. A reference may record a one-based source line, but file identity is more stable than line numbers.

A source or image must be explicitly listed before the localhost service can read it. Paths are canonical relative paths under docs/crates/apps/assets. Existing images retain their source provenance and classification; the reader never substitutes a concept preview for a runtime screenshot. No secrets, raw HTML, script, model endpoints or arbitrary remote links belong in this catalog.

Keep game-facing prose as missions; retain legacy serialized `quests` field references where compatibility needs them. See `docs/DEV_BIBLE.md` for routes, editing/check commands and exact verification scope. New facts must cite their implementation. Never promote work-only studies or future multiplayer content to implemented without actual integration.
