# Independent title-score source review

Reviewed original audio candidate `4edfd189d4e05f844e91a67a0f93f721800e0565` and Root's corrected audio source at `8727e7e0295095e338cf907b623174807d14e736`. The original candidate was based on `8c284637f3dfd4f86982f2dc01b10fb8a70c8162`. Only files beneath `work/title-score-independent-review/` were created by this reviewer. Root owns runtime changes and QA integration.

**Result: no unresolved blocking source defect found in the corrected audio source within this review's scope.** A concrete short-stall defect was found, reproduced, reported, corrected by Root and independently rechecked. This result is limited to source inspection, pure timing math and recorded WebAudio API behavior. It is not native DSP, listening, gameplay, mobile hardware, device output or a full build acceptance.

## Corrected defect and actual execution evidence

The original scheduler at `apps/game/src/audio.ts:749` tolerated up to 200 ms of overdue score debt. A title engine unlocked at time zero, followed by a 400 ms gap before the next scheduler tick, created eight sources with start times already before `currentTime=0.4`: `0.2222264067`, `0.3108396101`, and `0.3994528135`. WebAudio begins sources whose start time is in the past immediately, so this path bunched missed notes even though the existing 20-second-stall test passed.

Root changed the guard to skip whenever the next grid onset is before current context time, and expanded its actual scheduler-stall fixture to ambient/title arrangements with both 400 ms and 20 s gaps. The independent title-gap fixture now creates four future sources and zero overdue sources. The source change and tests are committed at `8727e7e0295095e338cf907b623174807d14e736`.

Executed independently on Node `v24.18.0`:

```text
node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs
exit 0; 49 tests passed

node --experimental-transform-types --test work/title-score-independent-review/edge-review.test.mjs
exit 0; 2 tests passed
```

The original independent edge run exited 1 with one failure and one pass. `edge-review-original-failure.log`, `focused-tests-original.log`, and `review-pins-original.json` preserve the original evidence. Final rerun output is in `focused-tests.log` and `edge-review.log`. `review-pins.json` records commands, exit statuses, source byte counts/SHA256 values, candidate commit blob hashes and evidence hashes. These logs are recorded API evidence, not generated audio.

## Source findings

- `MusicClock` correctly integrates tempo linear in beat space using the logarithmic integral of `60 / BPM`, and inverts that integral with `expm1`. The constant-tempo branch, boundary tempo, future-ramp replacement and reanchor path are consistent with the focused independent midpoint-integration tests. The transport retains at most 16 segments. Its callers use current/future beats or the retained frozen beat; it is not a general unbounded historical timeline.
- `AudioEngine.update` cleans and stores authoritative source state. `syncScoreMode` uses that intent's positive combat value, rather than the slowly blending audible combat layer, to request the combat ramp. Haven/site cleaning prevents an accidental combat ramp in those safe states. Chase ramps over four beats from the next whole-beat boundary. Release starts at the next whole-beat boundary plus two hold beats, then decelerates over eight beats. Thus a release has two hold beats **in addition to** the initial boundary quantization, not exactly two elapsed beats from an arbitrary fractional-beat HUD update. Re-chase replaces a pending release using the existing boundary tempo and transport phase.
- Title, hub and world arrangements use one context, scheduler, music bus, beat clock and harmonic phase. Continue changes the arrangement and requests the eight-beat slowing ramp; it does not replace the clock. Tests exercise PAN/RENKO/HUGO arrival at three title positions. Title/world blend uses a 0.4-second time constant; existing music voices receive a 0.24-second fade target and queued music at the arrangement boundary is canceled for replacement. Title returns cut gameplay effects and zero continuous engine/alarm mix intent; the rapid-state fixture verifies they do not restart under title.
- Future voices at or beyond a tempo change boundary are stopped, disconnected and removed before requeueing. Audible pads retime the outer voice fade, source stop and voice end to their beat boundary. Their individual source amplitude envelopes retain their original scheduled decay curves. Native renders/listening should inspect slowing Continue/release pad tails for gaps or weak tails; the recorded stop-time assertions do not measure the actual envelope after a deceleration.
- The active pools cap music at 32 voices and effects at 16. Each score voice owns a bounded source/node set. Final source `onended` releases a voice; release clears handlers, stops sources, disconnects all nodes and removes the voice. Mute preserves control authority. Suspension/disposal release voices and stop timers. The existing 30-second rapid-state fixture measures a maximum of 21 music voices and 12 future music voices within the 0.18-second lookahead. Pure clock stress covers 200 retargets. These are bounded API/resource observations, not heap profiling or all-day browser testing.
- Resume uses a lifecycle epoch and checks the newest intent after the asynchronous context resume. A running state event obeys `paused`, with only the explicitly bounded already-created death tail exception. Existing tests exercise old failed resume versus a newer successful resume, suspend during pending resume, interruption, repeated gestures, healing during interruption, stale death callbacks and disposal. The independent suspended-title-change test preserves beat and target tempo on resume. This is compatibility source/API coverage; mobile browsers and OS audio routes remain physically untested.
- The `TitleTheme` contract now contains authored note data, `enabled`, tempo and loop steps. The single Main URL guard in this base is correctly changed to `TITLE_THEME.enabled`; there is no remaining runtime title fetch/decode/recording-player path. The default constructor supplies the authored theme. The change requires QA preparation/allowlist support for `music-clock.ts` and `music-score.ts`, which Root owns and is integrating. No new shipped audio asset or service dependency is present in the reviewed runtime path.

## Frozen evidence and remaining acceptance

The original runtime/test files and handoff match their frozen pins after accounting for Git's Windows line-ending conversion. `main.ts`, `execution-evidence.json`, and `focused-tests.log` match their frozen raw SHA256 directly. `typecheck.log` has the same two-line text; the frozen hash is reproduced by its original mixed newline form (first newline LF, final newline CRLF), while Root's checkout uses CRLF for both. These conversion differences are explicitly retained in `review-pins-original.json`; they are not claims of equal raw file bytes. The two intentional Root changes are `audio.ts` and its stall test; all corrected reviewed paths match the corrected commit after line-ending normalization.

The frozen log reports 49 passes and clearly limits itself to pure transport/recorded API evidence. Its measured voice values are reproduced by the independent focused rerun. The original typecheck log contains only command/exit metadata, so this reviewer does not independently claim a TypeScript run. The reference file was not opened or rehashed by this reviewer.

The score notes remain explicitly authored, reference-inspired provisional data. The 512-sixteenth-step title form is 32 bars (about 45.37 seconds at 169.275 BPM), rather than the measured 42.68-second source form. Exact transcription, principal hook/octaves/bass/rest/cadence acceptance and recognizable-song acceptance remain open. The provenance explicitly records `exactTranscription=false` and `directListeningAccepted=false`.

No reference listening, native OfflineAudioContext rendering, physical output listening, browser session, gameplay run, screenshot, server, Spark worker, broad build, heap profile or device compatibility check was performed by this reviewer. Root's later DSP evidence must pin the corrected compiled source; this review does not inherit or certify that separate work.
