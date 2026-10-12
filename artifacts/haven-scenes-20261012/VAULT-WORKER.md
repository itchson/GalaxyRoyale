# Vault scene handoff (Spark worker)

## Files created (owned)
- apps/game/src/vault-scene.ts
- apps/game/src/vault-scene.css
- work/haven-scenes/vault-HANDOFF.md (this file)

No other files touched. No git operations. Root owns main.ts wiring, index.html/native scene wrapping, and commit.

## Public interface

```ts
export interface VaultHud extends CargoHud {
  components: ComponentHud[];
}
export function paintVault(target: HTMLElement, hud: VaultHud): boolean;
```

- Imports existing exports only: CargoHud + cargoRows from ./cargo, ComponentHud (type-only) from ./loadout. Side-effect import of ./vault-scene.css.
- paintVault returns true when it repainted, false when the cached snapshot key is unchanged (cargo.ts pattern; WeakMap keyed on target).
- Adds class "vault-scene" to the target so the stylesheet owns layout; Root should not add competing layout classes.
- Root supplies the full-screen scrollable container and close/routing; this module deliberately has no close button, backdrop, tabs, or panel chrome.

## Sections (view only, all values from HUD)
1. BANKED AT HAVEN - two-column ledger (MATERIAL / BANKED). Rows come from cargoRows(hud) filtered to banked > 0, so the named T1 subset double-count accounting (PHASE SILICATE / IRON FILAMENT / COOLANT KESSHO subtracted from T1) is reused exactly; no reimplementation.
2. UNBANKED SALVAGE - compact at-risk summary: single amber total line ("N UNIT(S) ABOARD") from cargoRows carried > 0, per-material carry counts, and one line: "Bank at Haven to keep it. Salvage is lost if the RAICHO is destroyed."
3. CRAFTED COMPONENTS - grid of owned (count > 0) components; art via div[data-pixel-kind="craft-component"][data-pixel-value=component.index] (the same accepted node kind/value loadout.ts uses, so native reuses real component glyphs), plus h3 name and "CORP / BANKED n" meta.
4. MISSION ITEMS - incomplete courier quests (event === "courier" && !completed), "COURIER PACKET / site / status" lines and retention note. Mirrors cargo.ts wording so retained-quest semantics stay consistent; no reward/capacity claims.

## Constraints honored
- No cargo/capacity fractions printed; cargo_capacity is intentionally not rendered anywhere. No "RAICHO slots" phrasing. No ship inventory slot UI. No MEISHI/currency content (upgrade currency stays on the MEISHI card per user decision).
- textContent only; no innerHTML anywhere. Semantic h3 headings with aria-label sections; h3/p/section/div only.
- Accepted native node kinds only: h3 and p are picked up by the pixel-ui generic text loop (root querySelectors "h2,h3,p,small,..." for menu cards) and craft-component icon div via the [data-pixel-kind] artwork loop, which skips collectionTextKinds so only the glyph renders. div wrappers are inert containers, matching the loadout.ts symbol()/section() precedent. No unknown data-pixel-kind values, no buttons/inputs, so buttonKind/textKind paths never see an unexpected kind.
- Snapshot change detection: key = [cargo_materials, cargoRows result, deliveries, owned components id/count]. cargo_materials is included only as a cheap stable scalar sentinel; it is not displayed. Scroll preserved across repaints (scrollTop captured/restored around replaceChildren).
- Responsive: sections stack in one column by default; @media (min-width: 640px) makes the four top-level sections a 2-column grid (same range as existing loadout.css breakpoints). Component grid is auto-fit minmax so it stays 1-column when narrow. Icons fixed 40px with min-width:0/flex-shrink:0 guards so rows cannot overflow. No 44px hit targets needed (view only, zero actions).
- No Canvas2D/image-div render path, no fabricated data: every count printed comes from the passed HUD via cargoRows or hud.components/hud.quest_log.

## Integration notes for Root
- Call paintVault(vaultContentEl, hud) when the floating VAULT hub opens the full-screen scene; pass a VaultHud built from the same fields used for paintCargo plus hud.components from your ForgeHud/flight HUD source (they already carry components: ComponentHud[]). The false-return snapshot path makes per-frame re-paint calls cheap while the scene is open.
- The vault target element should be the scroller; ensure it is reachable to the PixelUiBridge mutation observer. PixelUiBridge currently observes these ids: launch-screen, menu-screen, game-ui, debug-console, dialogue-screen, death-screen, droid-screen. The existing cargo tab renders inside menu-screen and is already covered; confirm the vault scene lands inside an observed root (or add its id to that list in pixel-ui.ts, which Root owns).
- The pixel bridge reads glyph state on a UI pass, so glyph changes may appear one frame late. If Root wants the icon glyphs to animate in on scene open (like the collection-entry reveal), tell me and I can add a small data-reveal hook; skipped for now to avoid extra motion.
- If Root prefers the salvage note phrasing softened or the risk color toned down, say so and I will adjust.

## Actual checks
- pnpm typecheck (tsc --noEmit, repo tsconfig includes apps/**/*.ts): PASS, no diagnostics (run twice, before and after final edits).
- pnpm exec prettier --check apps/game/src/vault-scene.ts apps/game/src/vault-scene.css: PASS after formatting these two files only (initial check warned; --write applied to owned files only).
- No browser run performed (worker constraints: no servers/browsers). Visual verification is Root''s on first full-screen open; recommend confirming: banked ledger values vs cargo tab, at-risk totals vs live cargo, component glyph art renders, and that nothing overflows the 2-column layout at ~1100px and single-column at 420px.
