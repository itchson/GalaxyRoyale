# Independent current Mote admission revalidation

The added four interior corners are a small, coherent positioning change. They improve actual moving-player emission in the tested 533×300 / zoom1.6 scenes without relaxing full-warning gates, physical clearance, sanctuary checks, dispatch limits, damage or phase timing. The 320×180 moving cases still emit no pulses during continuous reversals; all four resume after release. This is an admission improvement with a disclosed narrow-viewport limit, not a guarantee of uninterrupted firing against every movement path.

## Exact reviewed composition

Root's frozen `work/mote-current-integration` copies preserve the pre-edit controller SHA `53134b719dd2fb361cc5feb7fbbc020a3b446458605f4096b175d55e64fe8109` on the later actual Game source, including the accepted shared death helper, existing radiation audio, Scar and current ordinary enemy hooks. The baseline and candidate controller copies differ only by the comment and `.chain(corners.map(|corner| center + corner * 0.7))` before the original outer-corner chain. Absolute module paths in these work copies are review scaffolding, not a production patch.

By the time this review ran, root had applied the target addition plus a clarified two-line comment to production (`mote_controller.rs` SHA `1f8127aeb40d02402a11a17cc6a1e90485ea5314b364847349975d538341e881`) and strengthened the durable moving test. These two later source differences are explicitly retained in this review's pin. They do not silently replace the baseline or the earlier `work/mote-camera-review` evidence. The linked source modules/assets stayed identical during this independent compile and execution. The final post-check separately records a later root durable-test update: that root test file is not linked by these copied runtimes, which use their unchanged owned `tests.rs`. The original review pin is preserved, not refreshed to the new production tests.

## Safety and ordering review

Candidate enumeration is bounded at nine values: original preferred position, four interior corners, four original corners. Each value still passes the existing `legal_target`: finite/world bounds, intersection of current/predicted camera firing rectangles, swept player-disc clearance and swept expanded sanctuary-disc clearance. Distance sorting and deterministic tie-breaking remain; a currently legal retained target is not switched merely because an interior target becomes available. The four old corners remain available when the closer interior targets fail physical/path clearance.

The addition is a stack iterator over fixed arrays; it adds no collection or RNG consumption. The 0.125-world-unit corner inset remains before scaling, preserving the accepted far-coordinate conservative rounding protection. The helper still rejects a nonpositive firing extent. No change to position integration ownership, steer speed, turn limit, predicted inward-velocity removal, warning reset, locked attack gap, live emitter ownership, f64 charge/pulse clocks or postguard.

Actual warning checks still require the whole declared cue envelope under current and predicted follow cameras, plus the existing presentation margin. They run before/after steering and emission; unexpected later impulses beyond the reserved margin remain the documented limitation. Shared per-tick bullet reservations and final `spawn_bullet` cap are unchanged, as are charge-entry and actual-created-pulse audio ordering. Existing tests exercise full cap, destroyed emitters, ordinary shared swept damage/reflect/reward paths and sanctuary sweeps.

The interior corner is 70% of the legal half-extent relative to the current/end-camera midpoint. At the narrow high-zoom limit, this can fall inside the forbidden player/path disc, so it is correctly discarded. This explains why narrow motion still falls back to edge targets and warning cancellation. Do not weaken the safety gate to hide that tactical limitation.

## Independent actual execution

I independently compiled the **unchanged root-owned frozen copies** into executables under this review folder using the actual pinned engine_core/hecs/glam/serde libraries. No fixture preparation script was rerun and no candidate or production source was edited. The baseline eight-case matrix passes; the candidate's 23 actual radiation host/audio tests pass, including the appended matrix. Each matrix checks actual newly created hostile entities, post-tick interpolated actor and follow-camera bounds at alpha0/0.5/1, no death, and emission after input release. It does not infer firing from phase labels or manufacture pulse counters.

Each case first waits for a real charge, reverses full joystick every second for eight seconds, then releases for twelve seconds. Defense/regen ranks5 keep the declared scene alive; levelled means Scout XP1,320. All scenes use UI /1.6 as actual world viewport.

| UI | Scout | dt | Baseline moving emits | Candidate moving emits | Baseline / candidate first emit after release (s) |
| --- | --- | ---: | ---: | ---: | ---: |
| 533×300 | Fresh | 1/60 | 0 | 1 | 2.0333 / 1.9500 |
| 533×300 | Fresh | 0.05 | 0 | 2 | 2.2500 / 1.8500 |
| 533×300 | Levelled | 1/60 | 0 | 2 | 2.0833 / 1.8167 |
| 533×300 | Levelled | 0.05 | 2 | 2 | 3.1000 / 3.1000 |
| 320×180 | Fresh | 1/60 | 0 | 0 | 2.3500 / 2.3500 |
| 320×180 | Fresh | 0.05 | 0 | 0 | 4.5000 / 4.5000 |
| 320×180 | Levelled | 1/60 | 0 | 0 | 2.5000 / 2.5000 |
| 320×180 | Levelled | 0.05 | 0 | 0 | 5.1000 / 5.1000 |

Every case created four new hostile-emission ticks after release. The separate existing levelled moving fixture also reports128 visible cue ticks /2 actual emitting ticks at1/60 and58/2 at0.05. These are actual native fixture results, not software prediction, frame-time measurements or browser/phone evidence. No claimed physical-phone or human readability acceptance.

## Review outcome and remaining gates

No blocking source safety or dispatch-order defect found in the target addition. It is reasonable to compare this candidate in an actual foreground build while preserving the baseline and the broader narrow/human tuning task. Root's browser observation, actual minimum warning-cell visibility, moving paths beyond the fixture, camera/zoom transitions, physical phone behavior and later unexpected shake remain separate acceptance gates. The palette/tiny-cell studies must remain distinct from admission behavior.

Commands (read-only frozen inputs, output confined here):

```powershell
./work/mote-current-engine-review/build.ps1
./work/mote-current-engine-review/baseline-tests.exe radiation_enemies::tests::current_movement_release_matrix --exact --nocapture
./work/mote-current-engine-review/candidate-tests.exe radiation_enemies:: --nocapture
python work/mote-current-engine-review/pin.py
```

`source-fingerprints.json` and `evidence.json` retain original/candidate source identities, later production deltas, actual libraries, compiled test binaries and execution logs. No production/API/manifest/evidence files outside this review folder were changed.
