# Moving Mote admission: bounded tuning review

**Conclusion:** the zero-shot moving60Hz case is a temporary encounter-tuning limitation, not a stuck phase clock, lost projectile command or unsafe offscreen-fire defect. Current code deliberately cancels an incomplete warning when its conservative camera envelope ceases to fit. Human fairness has not been established; persistent motion suppressing a slow enemy should remain an explicit balance caveat, not be described as perfected combat.

Production and the accepted V2 candidate were unchanged. The prior composition report remains immutable. This work owns only `work/mote-camera-review/`. Copies of the actual frozen production Game were compiled with existing dependencies; the source pin remains core lib `ec2b3c897d4b5dc1ec70891d383f19a008fc0425a5dedd691c298cb41ddd3cea`. No new manifest, dependency, save field, renderer change or published evidence was created.

## Trace and smallest experiment

The original five candidates are the clamped preferred point and four firing-rectangle corners. In the original533×300/max-zoom/levelled-Scout case, full-joystick reversals cause two Charge→Waiting cancellations after approximately0.15s and0.40s of charge. Both recorded the current and predicted warning envelope outside admission. Neither sanctuary displacement nor loss of a legal target caused those cancellations. No projectile-cap or timer branch is implicated. The Mote's85-world-unit base approach is slower than this127.5-world-unit Scout; corner-bound placement leaves little room for the camera to follow the player during the required1.1s warning.

The experiment adds **four interior corners at70% of the legal rectangle half-extents** before the existing outer corners. Selection still chooses the nearest legal candidate; all five original candidates remain available. This changes the bounded candidate count from five to nine, without new state or tracking machinery. Existing player/path/sanctuary guards, current/predicted camera/shake admission, fresh charge duration, fixed gap lock, actual live emitter cells and pulse timing are unchanged. It does not shrink the warning envelope or allow clipped shots.

`tuning.patch` contains only that optional iterator addition and its comment. The compiled experiment also contains test-only trace hooks and local module-path scaffolding under this work directory; those are **not** proposed production changes. The70% value is an experimental starting point, not a proven optimum or a ship-speed rebalance.

## Executed actual-core evidence

```powershell
./work/mote-camera-review/build.ps1
./work/mote-camera-review/baseline/probes.exe camera_review:: --nocapture
./work/mote-camera-review/interior/probes.exe camera_review:: --nocapture
./work/mote-camera-review/interior/probes.exe radiation_enemies::tests:: --nocapture
python work/mote-camera-review/summarize.py
```

Both matrix tests passed, and the experimental actual-core suite passed **16/16**. These include all twelve stationary zoom/viewport admissions, actual pulse spacing, damaged emitter ownership, sanctuary body/projectile guards, shared caps, rewards and reused extraction. Other tests compiled but were filtered. This is native work-only evidence, not a new production regression run, GPU/phone measurement or browser playtest.

The matrix uses the **same original setup and charge helper** in an owned test copy, with test-only visibility widened to permit reuse. It performs eight seconds of the original full-joystick reversal pattern, then twelve seconds with the joystick released. It uses both unlevelled100-world-unit and levelled127.5-world-unit Scouts, R5 shield/regen, no fire, clear unrelated threats and two tick sizes. Actual Game controls, camera follow, integration, collisions, phase transitions and emitted hostile entities run normally.

| Logical UI at zoom1.6 | Scout speed | Step | Moving emitting ticks, baseline→interior | First stopped emission, baseline→interior |
|---|---:|---:|---:|---:|
| 533×300 | 100 | 1/60 | 0→1 | 2.03→1.95s |
| 533×300 | 100 | 0.05 | 0→2 | 2.25→1.85s |
| 533×300 | 127.5 | 1/60 | **0→2** | 2.08→1.82s |
| 533×300 | 127.5 | 0.05 | 2→2 | 3.10→3.10s |
| 320×180 | 100 | 1/60 | 0→0 | 2.35→2.35s |
| 320×180 | 100 | 0.05 | 0→0 | 4.50→4.50s |
| 320×180 | 127.5 | 1/60 | 0→0 | 2.50→2.50s |
| 320×180 | 127.5 | 0.05 | 0→0 | 5.10→5.10s |

An emitting tick means at least one real new hostile entity, not an inferred timer pulse. First-stopped offsets use the dispatch tick's start timestamp; add at most the listed step for elapsed end-of-tick time. All cases emit again after stopping; none died or lost its Mote during the20-second fixture. All sampled active warnings/emitting ticks pass the actual interpolated camera envelope check at alpha0/0.5/1 with5-world-unit shake allowance.

The original levelled533×300 case is reproduced exactly: **34 cue ticks/0 emitting ticks at60Hz**, and48/2 at0.05. The experiment yields128/2 and58/2 respectively. Its60Hz warning cancellations still occur, but later at approximately0.65s and0.37s; two real dispatches fit between them. It improves that declared case rather than guaranteeing continuous fire or all movement patterns.

At320×180/zoom1.6, the full conservative warning reserve leaves a vertical target half-extent of only about0.429 world units before the0.125 inset. Inner corners fail player-clearance admission, so the original outer-corner fallback behaves identically. The experiment does not manufacture room that the camera does not contain. The player can also deliberately ram the enemy in moving fixtures; post-collision separations near tangent with f32 rounding are not evidence that the approach guard prevents all player-driven contacts. The guards and collision path remain intact.

## Reproducibility and recommendation

`fingerprints.json` records the frozen source modules and owned copies. `results.json` retains all eight baseline/experimental traces and executable/log/source hashes. `baseline/original-moving.log` independently reproduced the unchanged original test before the experiment recommendation. Early observation logs remain explicitly named `trace-initial-invulnerability.log`, `trace-separate-fixture.log` and `trace-positive-zero-axis.log`; these are exploratory results, not the final matrix.

The exploratory matrix initially supplied left input as literal[-1,+0], whereas the original uses negated Vec2::X, producing[-1,-0]. Those exact half-turn inputs can take different floating-point heading/flight paths; replacing the expression with the original vector preserves the declared replay and matches34/0. The final matrix also reuses the original setup helper instead of treating a separately authored equivalent fixture as identical. No claim that every left/right trajectory has the same measured outcome is made.

**Keep the current release frozen.** For a later bounded tuning pass, the interior-corner addition is the smallest evidence-supported experiment from this review: one extra four-point candidate ring, no new ECS/controller state, all existing safety gates preserved. It warrants an actual browser/human A/B comparison on ordinary and narrow high-zoom scenes before adoption. Verify camera-shake and HUD/touch occlusion, sanctuary approaches, crowded player contacts, zoom transitions, reduced motion and sustained one-direction/diagonal movement. Include fresh/unlevelled and upgraded ships; do not approve solely from this one reversal pattern.

Do not lower warning duration, relax offscreen admission or speed up the whole enemy to hide this result. If narrowest-view moving pressure becomes a product requirement, it needs a separately agreed attack/telegraph envelope or camera-relative pursuit design with real acceptance evidence; this small target experiment intentionally does not solve that broader mechanic. Human combat pacing and physical Pixel6a/iPhone12 validation remain Pending.
