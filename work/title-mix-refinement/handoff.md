# Title instrument/mix refinement candidate

Branch `codex/title-mix-refinement`, based on Root `d53954c7956a2be71de34de12c4f309563bd8a40`, in `C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale`. The worktree was clean before switching, and `codex/title-score-engine` at `7c965e44ec1635d384eef6bee733e202fa1e9aa3` remains preserved. Production edits are limited to `apps/game/src/audio.ts` and `scripts/test/audio.test.mjs`.

The user listened to Root's first real native title render and requested **instrument sound and mix** changes. Root supplied prior native-render metrics for context: 48-second title, master0.75/music0.4, peak0.100266, RMS0.007335 (~−42.7 dBFS). The source mono reference peak0.5851/RMS0.0913 was unscaled. This pass did not render new PCM or claim a new RMS, perceived match or ear acceptance.

Changes:

- Title chip notes hold at full body through55% of their existing note gates, then release to the floor. A soft triangle on the same authored pitch/gate fills the fundamental. A3200→2100 Hz low-pass contour keeps the initial chip articulation while warming the held body.
- Title bass combines the existing triangle with a quiet same-pitch sine, using55% holds. The arpeggio is clearer and holds through45% of its existing gates.
- Bubble articulation now reaches its existing authored destination within90 ms /45% of the gate, while the held sine body is still present.
- Pads use slower150 ms inner attacks and sustained inner tone gains. A finite outer voice envelope holds the chord, releases over the final100 ms, and stops sources at the harmonic gate plus25 ms. On tempo changes, that envelope is evaluated before cancellation, its current gain is preserved, and its release/stop is moved to the new bar boundary. Repeated changes during release/recovery remain continuous; a very late change has at least15 ms to fade before its bounded stop. Inner pad tones no longer decay early at the original faster duration when the clock slows.
- Continue carries the existing title chord pad to its shared retimed harmonic boundary; other title notes retain the established fade. This remains one music transport and voice pool.
- Title snare/high ticks/glitch bursts are slightly lower, while the short kick has a little more body. The change adjusts title instrument balance rather than master/music buses.

The optional `hold` argument defaults to0 in `tone`/`attach`; world and effects still emit the original set→attack→exponential-floor envelopes. World/SFX/thrust levels, timbres, graphs and controls were not changed. Main, state, score/harmony, authored melody/octaves, tempo data, clock/stall correction, manifests, QA helpers, studio and renderer files were untouched. No PCM asset, recording playback, server, browser, model or worker was added.

Actual final verification on Node24.18.0 / TypeScript5.9.3:

```powershell
node node_modules/typescript/bin/tsc --noEmit
node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs
```

Both exited0; all54 focused tests passed. Raw logs are `typecheck.log` and `focused-tests.log`. `execution-evidence.json` records scope and outcomes; `source-pins.json` pins tested source/evidence bytes and checks unchanged baseline modules. Existing lifecycle, one-context, mute, chase/Continue, short stalls,32/16 budgets, cancellation and cleanup tests still pass. The30-second recorded-API stress fixture again measured maximum21 music voices /12 future music voices inside the0.18-second queue. Those are API/resource observations, not native DSP measurements.

Five new meaningful tests verify held chip/bass body and bounded release/cleanup, inner-pad sustain after real Continue deceleration, gain continuity through repeated pad gate changes, unchanged world/cue gain curves, and bubble destination timing while its held body is present. The measured first lead half-gate gain is0.03822 for a0.177226-second gate; this is scheduled gain automation, not an RMS or loudness result.

Remaining gates: Root's fresh actual OfflineAudioContext title/Continue/mute renders with hashes and peak/RMS/nonfinite checks; listening comparison for instrument body, balance, pad carry and transition comfort; physical output-device review; perceived reference-song match and exact transcription remain provisional. This candidate is ready for that native render/review, with no audio acceptance claim.
