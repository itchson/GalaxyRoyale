# Independent title instrument/mix source review

Reviewed Root audio candidate `51b529f4c82c1abb8a0c8bfb386263c5c81cc1cc`, integrated into the frozen check source at Root `d028f0d7a3c589b5896f536f08e76aa504cb2923`. The author candidate was based on Root `d53954c7956a2be71de34de12c4f309563bd8a40`. This reviewer created only `work/title-mix-independent-review/**`; no runtime, shared source, manifests, commits or earlier review artifacts were modified.

**No concrete blocking source defect found in this bounded title-only review.** The source addresses the previously identified early-decaying title-pad limitation by sustaining its inner gains and moving the finite gate to the outer voice. Approval of instrument sound, mix comfort, waveform quality and reference fidelity remains open. This review contains source and recorded-API evidence only.

## Actual verification

On Node `v24.18.0`, these commands independently exited zero:

```text
node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs
54 tests passed, 0 failed

node --experimental-transform-types --test work/title-mix-independent-review/edge-review.test.mjs
4 tests passed, 0 failed
```

Raw output is preserved in `focused-tests.log` and `edge-review.log`. No test in this independent pass failed. The four additional checks cover actual Continue during an inner pad attack, Continue during the final 40 ms of its outer release, hard suspension during a deferred title resume, and 20 seconds of repeated title/Continue/chase changes. The recorded harness was copied into this review's owned directory from the earlier independent review.

## Envelope, transition and lifecycle findings

- `attach` leaves a title pad's inner gains at their post-attack level. The title-pad branch contains set→linear attack automation, with no fixed early inner exponential decay. `titlePadGate` supplies an explicit outer release to `FLOOR` and a source stop 25 ms after the gate. A normal pad still has only four inner sources and a finite harmonic gate; it is not a persistent voice.
- At creation the pad's harmonic end is step +15 sixteenths, before the next 16-step chord boundary. The outer pad gate holds its level, then releases linearly over the last 100 ms. Its inner attack is 150 ms. The production title tempo/gate durations keep the initial outer automation points in chronological order.
- On a tempo change, the outer gate evaluates its own piecewise-linear envelope at current context time before cancellation. It re-establishes that instantaneous value and changes the gate and stop deadline. A long extension recovers toward the pad's existing level over 25 ms; a short remaining gate continues toward the floor. The `end-now>0.13` branch leaves all recovery/release points ordered, and the minimum 15 ms terminal gate prevents an immediate late stop. At most four gate points are retained on each pad.
- The independent real Continue check during its inner attack moved every stop from `1.3991980505` to `1.4514851242` seconds. Sources survived the former deadline and were stopped/released/disconnected at the replacement deadline. The focused Continue test also checks sustained inner gains beyond the old faster gate. Repeated `stop(newDeadline)` calls are replacements while those sources are still running; the implementation never asks an ended source to revive.
- The independent Continue check during the original pad's last 40 ms preserved its calculated gain `0.28006` and chose the two-point terminal release, so that late release did not regain full level. The focused repeated-gate test verifies the same instantaneous-gain continuity through release/recovery retiming and the minimum terminal fade. These calculations inspect scheduled automation and API calls, not emitted samples.
- `setTitleScreen` exempts the held title pad from the generic music fade, allowing that one existing chord to reach its shared harmonic gate. Other already audible music notes retain the established fade. Future music voices remain invalidated/requeued at the arrangement boundary. The original beat clock, tempo ramps and short-stall fix are unchanged.
- The optional hold argument defaults to zero in both `tone` and `attach`. World and SFX callers therefore keep the original set→linear attack→exponential floor envelope path. Source inspection confines nonzero holds and new chip/bubble instrumentation to title helpers. The focused comparison test verifies the default envelope sequence for actual world voices and a real mote-charge cue. Global music/master/effects levels, the shared graph and thrust implementation are unchanged.
- Additional chip body and same-pitch sine bass sources remain inside their existing music voices. The voice pool caps remain 32 music/16 effects. The existing 30-second fixture still measures maximum 21 music voices and 12 future voices. The independent 20-second repeated-state fixture measured maximum 17 music voices and 56 active music sources; gates remained finite and point lists bounded. These are sampled active API-resource measurements, not heap profiling or all-day execution claims.
- Final-source completion still decrements pending sources, clears all handlers, stops sources, disconnects nodes and removes the voice. The new held pads follow that normal cleanup path. Suspension, disposal and future-voice invalidation remove them. The independent deferred-resume test released the held pad, cleared the scheduler and rejected stale scheduling after a newer hard suspend. Existing mobile/interruption/death-tail lifecycle guards are unchanged and their focused tests still pass.

## Pins and scope

`review-pins.json` records exact Root working-file bytes and SHA256 values separately from canonical Git blob bytes/hashes. The author pinned its final working bytes; Root uses Windows CRLF conversion. The audio source and test's canonical commit/LF bytes match those original author hashes, while their raw Root working hashes differ solely by line endings. Candidate source and current frozen Root source match after canonical newline normalization. The unchanged baseline modules are compared independently to the author base commit. Handoff, execution metadata and test/typecheck logs are also checked, including the typecheck log's original mixed newline form. No claim treats normalized text equality as raw byte equality.

The frozen author log reports 54 passes; the independent focused run reproduces them and the prior resource values. The author typecheck log contains command/exit metadata, so this reviewer does not claim an independent TypeScript execution. Supplied prior DSP metrics and the user's earlier listening selection provide task context only; this reviewer did not inherit them as new acceptance evidence.

No native DSP render, browser, server, game session, screenshot, broad build, Spark worker, physical output device listening or reference audition was run here. Root's fresh OfflineAudioContext render must pin these corrected compiled bytes. Ear acceptance, exact transcription and perceived reference-song fidelity remain open; authored note/octave/harmony/tempo data were not changed by this title instrument/mix pass.
