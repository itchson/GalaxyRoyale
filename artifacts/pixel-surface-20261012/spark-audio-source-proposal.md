

## Worker handoff — audio source audit (read-only)

**HEAD at start:** `303659e` (`303659ec252f4ab7bded3c137ecec5ca3ca33edb`, "Vary seeded ringed-planet group orientation"). Reads used: 5 shell calls (git+sizes, AGENTS.md+AUDIO_REVIEW.md, audio.ts, audio-state.ts+grep, grep verify). No edits, no listeners/builds started, no secrets inspected, no recursion.

### Already handled by current source (do not re-fix)

- **Calm thrust bed:** `continuous("engine")` uses a sine pair on the shared A–E fifth (`ENGINE_NOTES [45,52]`) under a 260 Hz/`Q 0.5` lowpass with a tiny 0.008 noise air bed at `ENGINE_LEVEL 0.24`, `setTargetAtTime` smoothing (0.18/0.24 s), plus four-second continuous-breath LFO — genuinely quiet and low; it sits *below* the score, not over it.
- **Music sync:** one 112-BPM shared clock (`schedule()` 25 ms tick, 0.18 s lookahead, 0.2 s lag hard-reset), per-`beat` drum/alarm resets, all layer gains via `setTargetAtTime` — no `setTimeout`/`setInterval` scheduling.
- **Bus smoothing:** `advanceMix` uses one exponential `toward()` approach with per-metric taus (engine 0.08/0.16, combat 0.45/1.6, alarm 0.12/0.25, theme 1.4, space 1.1); `max(0,…)` guards prevent `NaN` runaway; `audibleThemes` caps to 2 crossfading themes.
- **Dynamic biome/home/battle music:** already implemented — biome 1–5 → themes 1–5, biome 0 → Expanse (14), corporation-select → PAN/RENKO/HUGO (15–17) via `advanceMix` theme slot, champion → 6+champ, and battle layers gated on real `mix.combat`/`mix.boss` in `scoreStep`. The user's "dynamic biome/home/battle music" ask is largely **done**; only the blends below have measurable gaps.
- The pinned lib (`lib.dom.d.ts`) contains `BiquadFilterNode/Type`, `createBiquadFilter`, `createWaveShaper`, `setTargetAtTime`, `getChannelData` — the source's spellings match the DOM lib; no shim, no API bug, no false alarm.

### Three source-backed issues (real, not unmeasured-hearing)

1. **Thrust ignores the dynamic space field** — `advanceSound`/`continuous("engine")` shape only on `mix.engine`; the noise bed is hard-coded `0.008` and the master lowpass is fixed, while `musicWet` (0.12) never changes. In Echo (`space`→1) exploration music is wettest yet the thrust stays fully dry and forward — the one thing stopping a real blend. Matters: the "calm thrust under exploration" balance breaks by biome.

2. **Scheduler phase hazard on drift reset** — `schedule()`'s lag guard (`nextNote < currentTime - 0.2`) resets `nextNote = currentTime + 0.02`, discarding grid phase rather than re-quantizing to the 112 BPM grid; the 112-BPM `buildGraph` echo and the four-second breath LFO use raw `context.currentTime` sine, so both fall out of alignment with the note grid after a reset. Matters: a single stall mid-exploration audibly hiccups the delay/breath.

3. **Hidden silent path: `engineOscillators` nested inside the noise guard** — `this.engineOscillators = oscillators` is assigned only at the end of `if (kind === "engine" && this.noise)`. If `this.noise` is null, `advanceSound`'s speed/throttle bend (`setTargetAtTime` at ~L874) silently no-ops while the sine pair keeps running at the stale `frequency(ENGINE_NOTES[i])`. Matters: the thrust "engine pitch rises with throttle" behaviour dies with no error path; fix is to hoist the assignment out of the `if`.

One caveat flagged only: `finishDeath`'s instant music-bus gate (0.008 s ramp) is intentional, but it is the only hard-gain discontinuity left in the file and is unmeasured by any existing capture.

### One lean next implementation

Scale the existing engine voice with the space field it already receives: in `advanceSound`, multiply the noise-bed `GainNode` and the master lowpass cutoff in the engine `continuous()` path by `this.mix.space` (no new nodes, no new bus — reuse the existing `mix.space` and `musicWet`).

### Acceptance via existing `OfflineAudioContext` helper

Add one **sixth declared fixture** to `scripts/qa/audio-review`: a thrust-blend solo — throttle ramp `0→1` over 4 s at Echo (biome 4, `space`→1, `hull` 1, no hostiles), then 2 s release, current buses `.75/.4/.75`. Assert through the real master chain (`limiter`→`ceiling`): `over_unity === 0`, zero nonfinite, the eight 4096-sample FFT windows show engine-band RMS in the same pocket as the representative fixture's 0.295 peak, and the new-vs-quiet phase reports per-phase PCM — validating the blend at every biome rather than one fixed value. Report/float-WAV written once (`wx`) under `work/audio-review/`; no new dependencies, no AI music, and any "calmer/more present" verdict stays a `humanIntent` ear-check, not a fabricated result.