# Frozen boss gameplay/documentation review

Read-only review, 11 October 2026. Checked `docs/BOSSES.md`, `README.md` and `docs/ROADMAP.md` against the frozen core. No production edits or repeated tests.

The documented fifteen encounters, five-tier binary promotion, 24-base-kill reservation, surplus credit, highest-tier priority, retry behavior, schema-3 migration, 25/35/45/55/65 boss XP and fourth-eligible-defeat signature guarantee match the implementation. Evolution derives from the highest defeated family generation; Sovereign has intrinsic E5 and virtual rank five without overwriting global ranks. Coil outward initialization and conservative contact versus exact occupied-cell projectile collisions are described accurately. README/ROADMAP explicitly leave natural progression, human balance and physical-device acceptance pending.

Two wording inaccuracies in `docs/BOSSES.md:3` merit a later documentation correction:

- “Every encounter has a visible charge” guarantees more than the source does. Every boss has a charge phase and reactor/HUD cues, but the phase begins while a naturally spawned head can still be offscreen. Diagnostics also place heads 210 units above the player. Suggested wording: “Every encounter has a charge phase with reactor and HUD cues.” First-charge readability still needs natural browser play review.
- “Low hull accelerates firing by 15%” is numerically imprecise. `bosses.rs:527` multiplies firing intervals by 0.85, meaning **15% shorter intervals**, approximately 17.65% higher frequency. Use the interval description.

Optional clarity in `README.md:31`: XP drive is additive relative to base (`1 + 0.025 × (min(level,12) − 1)`), not compounded. Level twelve gives an unpurchased Scout speed 127.5 and acceleration 331.5. Gates restrict the next purchase; acquired basic hardware stays global and remains effective on a low-XP newly selected ship. Only signature rank strength additionally applies that ship's level cap. XP levels above twelve do not add more drive or hardware ranks.

## Remaining Scout balance risks, with existing evidence

The existing 45-case study is in `work/scout-balance-review.md` and `work/scout-balance-results.json`. It used real CPU combat at 60 Hz, exact telemetry aiming or stationary input, isolated diagnostics, no ordinary waves, and no human/browser/device acceptance.

- **First natural boss is usually above L1:** the same-Scout prerequisite floor is G1 L4/120XP, G2 L5/290, G3 L8/650, G4 L12/1,390, G5 L17/2,890. Switching ships or legacy pending saves can violate that floor. Do not interpret an L1 G5 loss as normal onboarding evidence.
- **Coil and combined movement/fire:** the exact-aim/no-dodge, no-purchased-hardware floor bot defeated nine of fifteen bosses but lost all five Coils and Bastion G5. Coil G1 killed it after 12.07 seconds; actual first incoming damage was at 2.60 seconds. This flags human weaving, tail targeting, core access and joystick precision for testing. The bot's center shots can miss hollow silhouettes and it does not deliberately attack segments, so these losses do not justify a difficulty change by themselves.
- **Purchase pacing versus XP:** before missions/boss rewards, ten points in one specified category average 150 base kills or 120 rock kills. The first 24 same-family base kills yield only 1.6 expected points per category. Drive/level gates can progress before a protective purchase becomes affordable. Actual point collection, sanctuary events and player mining choices must be measured.
- **Travel and initial hull:** one default-seed full-right launch without firing reached the belt at 7.23 seconds and combat at 14.63 seconds after three asteroid contacts, leaving 76 hull. This is a single observed path, not a population average or a proposed target.
- **Framing and retries:** first charge may occur offscreen, the long Coil chain can exceed a short viewport, and a retained higher-tier queue gets priority on another expedition. The confirmed immediate tail-contact bug is fixed and covered by the existing 63-test pass; those remaining framing/progression choices still need human observation.

Next evidence should prioritize fresh natural expeditions, a legal early Scout versus Coil G1, first visible charge at small landscape views/zoom limits, and time to the first actual dock purchase. Use ordinary waves and real input, record fixture grants separately, and retain desktop/phone evidence boundaries. No new tuning recommendation or publication-blocking source defect from this review.
