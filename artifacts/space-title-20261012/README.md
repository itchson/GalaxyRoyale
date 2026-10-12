# Space detail, starter vitals and sports-league title

This is a bounded presentation and native-art integration, not a release or a
claim that the full production plan is complete. The final source revision and
raw evidence hashes are recorded in `manifest.json`.

## Delivered behavior

- Factory hull health is five 20-HP chunks. Real capacity upgrades add full or
  partial chunks within the existing curved track. HUGO's starting shield
  remains three 10-HP chunks; PAN and RENKO gain no innate shield.
- Each asteroid authors a deterministic 64-grid body at birth. Rock, metal,
  ice and porous families have coherent shapes and tier palettes. The first
  decorative streaks were rejected by the user and replaced with compact
  mineral deposits. Initial pits remain opaque; projectiles still remove real
  occupied cells. Haven's no-XP / T1 training rewards are unchanged.
- Deep nebulae use veils, billows and fine emission filaments with four relative
  transforms: 2,128 cells at full detail and 538 at reduced detail. The shared
  soft material, 0.08 parallax and existing 8,000 / 4,000 cosmic caps remain.
- GALAXY ROYALE is now explicitly the universe's premier arena-battle sport.
  The original sports-league logo uses athletic lettering, orbital arena rails
  and a gold award crest. Its 44,985 native cells use nine transforms; lettering
  stays still while restricted lighting and the award face animate. Reduced
  Motion freezes the authored animation. It is not a PNG rendered over the UI.
- The title layout keeps measured controls and the 8:3 mark within wide,
  short-landscape and narrow viewports. The optional original theme loader uses
  the existing AudioContext and volume buses. No theme file is shipped while
  awaiting the user's recording.

## Actual evidence

`title-handoff.md` and `title-independent-review.md` describe the final sports
identity. `assets/ui/title-mark/provenance.json` retains the generated reference
and declared conversion losses. The earlier crystal wordmark remains in Git
history; it is superseded by the sports-league direction.

`asteroids-software.png` is the corrected native-packet contact sheet.
`nebula-software.png` is a software approximation of actual native packets.
These are not GPU screenshots. Their original packet sets and detailed source
pins remain in the specialists' work directories identified in the handoffs.
Browser captures contain JPEG bytes and use `.jpg` filenames. The historical
`asteroids-webgpu.png` alias retains the same JPEG bytes only because an earlier
append-only Studio review already linked it; new evidence uses the correct name.

Actual browser captures are labeled by renderer. The QA session uses an
isolated profile, AHR#DE4FD6CC1F421C7F, with world seed 1145625092. The gas was
reached using the public QA command `warp 30274.8516 8880.8115`; scene rewards
were disabled. `scene belt` was used for the asteroid capture. These images do
not represent naturally earned progression or a completed training run.

The final title captures use the fresh `league-final.localhost` QA origin and
cover WebGPU at 1280×720, 848×390 and 390×844, and WebGL2 at 1280×720. The
short-landscape screenshot may report 847 physical pixels after
the browser's viewport rounding. Renderer console captures independently
identify the selected backend. Gas captures cover both renderers; WebGL2 also
covers zoom 0.6 and 1.6 with reduced-detail/Reduced Motion controls. Normal zoom
1.0 and normal detail/motion were restored after inspection. The final title
also has an actual WebGL2 Reduced Motion capture; its settings toggle was
restored and the temporary viewport override reset.

The earlier gas capture with the default proof seed placed this profile's cloud
offscreen. It was replaced with the correct seeded position. Source inspection
confirmed the camera/parallax transform, rather than changing runtime behavior
to match incorrect QA coordinates.

The existing PWA update flow was exercised through its visible Apply Update at
Haven control. This is a desktop interaction check, not full browser/PWA release
certification. One owned preview is retained; no specialist starts a game server.

## Validation and limits

Final `*-sports.log` files record **701 native tests passed / two explicit
exports ignored**, **434 shell tests passed / one existing Windows skip**,
**seven exact conversion tests passed**, strict workspace/all-target Clippy,
and the normal TypeScript / dual-WASM / web / PWA / Studio build. No failures.
Earlier `*-final.log`, `*-all.log` and `*-checked.log` files are historical
checks from before the sports identity revision. Runtime source is
`d85e82a8f9c8fee1d05b9b7ee54f1e3d1219337f`; world captures preceded that
title-only revision, at `f8e2f65511a52bd32261f50bad7131801ed1cf6b`.

The complete title fixture retains every declared component at 48,894 cells
and 157 transforms. This is a native fixture, not a browser frame measurement.
The independent audio review found two real lifecycle issues, now fixed:
visible-title playback after graphics recovery, and audio suspension when the
page becomes hidden before game initialization completes. Audio tests record
Web Audio API behavior; they do not establish subjective sound quality.

Pending: the original title recording, audible loop/transition approval,
physical-device compatibility, 30-minute frame-time/memory soak, broad browser
coverage, natural training/campaign completion, and final subjective art approval.
The encoded/decoded audio limits do not prove a peak decoder-memory ceiling.
The asteroid foreground budget is separate from the cosmetic gas budget; no
performance claim is inferred from a still screenshot or short FPS sample.

The Spark audit was interrupted without a returned report and receives no
review credit. Independent OpenAI specialist reviews and actual Studio run
records identify the work that was completed.
