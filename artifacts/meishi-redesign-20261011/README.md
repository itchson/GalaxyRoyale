# Portrait-led native MEISHI redesign

Runtime tested: `f3a723081033a2cdd9bab15e37241fe334530716` on `codex/testing`.
The tested preview receipt pins source and package hashes. The approved concept
remains `../concepts/meishi-collectible-20261011/pan-collectible-reference.png`;
the previous `meishi-collectible-20261011` implementation was rejected visually.

This pass enlarges the portrait, removes duplicate combat statistics, keeps the
current RAICHO and level, and introduces cached metallic fittings, corporate
inlays and eight octagonal wells. Close remains the only card button. Native
glyphs now honor measured center/right alignment, and small hull icons retain
continuous fractional fitting. Live statistics remain in the separate RAICHO menu.

## Actual validation

- Workspace Rust tests: 680 passed, one previously ignored, zero failures.
- Strict workspace/all-target Clippy, TypeScript and production build passed.
- Shell tests: 425 passed, one existing Windows symlink skip, zero failures.
- Native nine-profile complete-card fixture: worst 53,402 / 60,000 cells.
  The unchanged quota charges all submitted occupied cells, including clipped
  work. Reduced-motion and exact art-cost checks are retained in engine review.
- Actual in-app-browser WebGPU and WebGL2 PAN/DORA/RENKO captures at nominal
  1280x720, 848x390 and 320x700. Both backends were confirmed using the public
  console `renderer` command. Browser landscape PNGs measure 847x390 despite
  the 848-wide viewport override; this capture rounding is not a phone result.
- All identity, points, eight empty wells and Close are visible. At short
  landscape, the two-column layout has 273px client/scroll height and keeps
  the content one CSS pixel below Close. Narrow portrait has no content
  overflow. Keyboard focus cycles between Close and the scrollable card.
- All eight earned emblems were checked using a supplied QA save restored
  through Settings / Save & Backup, then reloaded across backends. This is
  display/save evidence, not naturally earned campaign completion.

The initial hand-edited all-eight fixture was correctly rejected because its
champion ability blueprint flags did not match its patches. Gameplay corrected
only those eight flags and validated the fixture through the actual storage
module. See `qa-fixture/`. The final desktop all-eight captures and DOM record
are explicit supplied-state evidence.

The first immediate-resize WebGPU screenshots caught an unsettled browser
capture. The final WebGPU 848/320 files were recaptured after layout settled
and visually inspected; they are complete. `layout-*-before-fittings.png` are
historical intermediate comparisons only. The native software contact sheet
is a software approximation, not a GPU screenshot.

## Review and remaining work

Actual independent Art, Engine and Gameplay/UX reviews are retained in their
directories. Art found no confirmed clipping blocker but identifies real
style gaps: closer portrait framing, more sculpted metallic masses and
stronger socket recess planes. User approval applies to the generated concept
and new hub, not this final card. Whole-game visual consistency, other live
corporation/type combinations, physical phones, broad browsers, natural
campaign completion and a 30-minute GPU/memory soak remain separate gates.

No new image-generation call occurred in this pass. Fittings are separately
authored native geometry informed by the approved reference. The game remains
independent of the Studio service. Exactly one owned preview was independently
confirmed by the evidence audit.
