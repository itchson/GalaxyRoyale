

Review complete — read-only inspection of `meishi.ts`, `meishi.css`, `style.css` menu scaffold, and the `main.ts` tab/bridge code. Two concrete findings (one scoped blocker), plus the stable layout math behind it.

## Finding 1 (scoped blocker): the meishi heading row reserves no gutter for `.menu-close`, and the ≤390px query removes the only right buffer

**Mechanism.** The close-hit gutter is reserved only for the generic menu header: [style.css:277–281](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/apps/game/src/style.css) applies `width: calc(100% - var(--ui-hit) - 8px*uiScale)` to `.menu-card > .eyebrow`, `> h2`, `> #menu-description` — the *direct children*. On the meishi tab those are hidden ([main.ts:993–995](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/apps/game/src/main.ts)) and content lives inside `#meishi-content > .meishi-card`, whose top row `.meishi-heading` ([meishi.css:29–33](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/apps/game/src/meishi.css)) has no right reservation at all. The only buffer is `.meishi-identity { padding: 16px 18px 0 0 }` (meishi.css:38) — and the portrait query zeroes it: `.meishi-identity { padding-right: 0 }` (meishi.css, ≤390 block, last rule).

**Computed overlap at 320×568** (max-width:390 query active; base 4-col layout since 568 > 560):
- Card = `min(500px,100%)` → 320 CSS px (no safe insets); `padding-inline:18px` → content box 18…302.
- Close button: `.meishi-surface .menu-close { top:7px; right:7px }` (meishi.css:13–16) over `icon-button { width/height: var(--ui-hit) }` (style.css:678–682), `--ui-hit = clamp(44px, 18px·uiScale, 52px)` (style.css:23). Worst-case 52px → occupies x ∈ [261, 313], y ∈ [7, 59].
- Identity column = 320 − 18 − 112(portrait frame) − 10(heading gap) = **160px wide, wrapping at x = 302**, i.e. 41px of wrap width (261→302) sits under the button. Title h3 starts y ≈ 26+6 = 32 (card+surface padding) and its first line runs 32→56, fully inside the button's 7→59 y-band. A ~100px string like `MEISHI CARD` ends near x ≈ 250, but `LEGACY PILOT`/issuer lines at 11–14px can reach the 261 boundary with the 18px buffer gone. Text uses `overflow-wrap:anywhere` (meishi.css:35–37), so it wraps at 302, not around the button → glyph collision is possible, not guaranteed.

**Landscape variant of the same bug (848×390, 601×560):** the 2-col card puts `.meishi-raicho` in col 2 under the close. Its only guard is `padding: 12px 28px 0 0` (meishi.css, landscape block), but required clearance is `7(inset) + 52(max hit) = 59px`. 28px reserved vs 59px needed → ship name/vitals can run under the button for long strings (`RAICHO UNAVAILABLE` / `HULL 100/100 / SHIELD 100/100`).

**Smallest fix:** one declaration in each context — add a close-gutter right padding to `.meishi-heading` (portrait) and raise the landscape raicho right padding to `calc(var(--ui-hit) + 12px)`; e.g. `.meishi-heading { padding-right: calc(var(--ui-hit) + 14px); }` and restore a nonzero `.meishi-identity { padding-right }` in the ≤390 block. No markup change needed in `meishi.ts`.

## Finding 2 (no blocker): 8-socket grid and vertical budgets compute clean at all five viewports

Verified against source (CSS px = viewport px; these queries key off the CSS viewport, not `--ui-scale`):

| Viewport | Active query(s) | Patch cols / slot | Verdict |
|---|---|---|---|
| 480×700 | none (base) | 4-col, (432−24)/4=102 ≥ 64 well | fits |
| 848×390 | landscape | 8-col, (772−56)/8≈89.5 ≥ 48 well | fits |
| 601×560 | landscape (inclusive bounds) | 8-col, (553−56)/8≈62 ≥ 48 well | fits |
| 1280×720 | none (base, capped 500px) | 4-col, (452−24)/4=107 ≥ 64 | fits |
| 320×568 | ≤390 | 4-col, (284−24)/4=65 ≥ 56 well | fits |

`minmax(0,1fr)` + `max-width:100%` on the well prevents blowout below these widths; landscape stack height ≈ 326px vs ≈ 342px available at 848×390 (tight but `#meishi-content` has `flex:0 1 auto; min-height:0; overflow:auto`, meishi.css:10–13 + style.css:310–318, so it degrades to scroll, not clipping).

## Uncertainty

`--ui-scale` is set from `innerHeight / galaxy.viewport()[1]` ([main.ts:638](C:/Users/mitch/Documents/ChatGPT/galaxy-royale/apps/game/src/main.ts)); exact per-device values can't be derived statically, so I bounded the close button to the CSS clamp range 44–52px — the Finding 1 overlap holds across that whole range. Per instruction I did not re-raise the native patch-extraction→continuous-scaling path (engine agent owns it). No files mutated, no servers, no browser used; all numbers above are computed from the cited CSS lines.