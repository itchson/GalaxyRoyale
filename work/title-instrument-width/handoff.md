# Title instrument width candidate

Branch `codex/title-instrument-width`, based on Root `d028f0d7a3c589b5896f536f08e76aa504cb2923`, in `C:/Users/mitch/.codex/worktrees/galaxy-pixel-art/galaxy-royale`. The worktree was clean before switching; `codex/title-mix-refinement` at `a2ad83da64bb74c2a8ca469c2c3e5444a1a375a8` remains preserved. Only production `audio.ts`, its focused test file, and this handoff directory changed.

**Root integration requirement:** add `"createStereoPanner"` to the creation-method Set in `scripts/qa/audio-review/review.js`. Its real OfflineAudioContext Proxy must bind/register that new node factory, as it already does for other creation methods. No new source module is needed. This specialist did not edit the shared QA helper.

The user listened to the second real native draft and said it still needs fuller instruments. Root supplied V2 context: 48-second title at master0.75/music0.4, peak0.156361/RMS0.027731, seven clean native fixtures. This pass targets layered instrument color/width; it did not generate new native PCM or claim improved RMS, perceived match or ear acceptance.

New title-only patches:

- Lead: exactly3 finite sources per note — filtered pulse pair at+3/−4 cents, panned−0.28/+0.28, plus the existing warm triangle at the nominal authored pitch. The pair has slightly different filters and9/14 ms attacks, while the nominal body attacks over12 ms. Gates/holds/releases stay authored and finite; no extra octave, LFO or delay source is used.
- Pad: exactly8 finite sources — four nominal triangle chord tones plus a quiet filtered saw layer per tone at±4 cents. Moderate opposing pans are at most0.32. The two layers have180/230 ms attacks and darker1200 Hz harmonic filters. Layer levels are rebalanced for the stereo route, while the existing sustained inner gains and retimed outer pad release/stop remain intact.
- Bass: exactly3 centered sources at the same authored fundamental — triangle, sine and a restrained520 Hz filtered saw body. The scalar peak-level allocation remains0.095 across the three layers (formerly0.063+0.032, now0.051+0.032+0.012). Attacks are8/10/14 ms; no stereo sub-bass or new pitch is added.
- Arpeggio and bubble components receive gentle alternating pans (±0.14 and±0.12). Existing percussion/noise, note data, loop form, tempo, beat/harmony phase and user buses remain untouched.

Panning is explicitly opted into only by title components, after their source gain envelope and before the existing Voice gain. If `createStereoPanner` is absent, every layer keeps a finite mono route. There is no inversion, channel delay or modulation oscillator. Each filter/panner/source belongs to the same Voice and disconnects on end, cancellation, hidden/suspend or disposal. The default pan0 path creates no panner and preserves world/SFX/thrust graphs and envelopes.

Bounds: lead3 sources, pad8, bass3, arp/bubble1 each. An authored compound backbeat voice can contain at most9 sources (lead3+arp1+bass3+kick1+snare1). Existing32-music/16-effects voice limits and0.18-second lookahead remain unchanged. The30-second rapid-state recorded-API fixture measured maximum21 music voices,12 future music voices,71 live owned music sources and210 live owned music nodes. These are actual API/resource observations, not CPU measurements or native DSP loudness.

Final verification on Node24.18.0 / TypeScript5.9.3:

```powershell
node node_modules/typescript/bin/tsc --noEmit
node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs
```

Both exited0;58/58 focused tests passed. The existing short-stall, lifecycle, Continue/chase, pad retiming, mute and release tests remain passing. Four new tests verify bounded detuned stereo lead/gate/cleanup; bounded pad/bass layers and register; missing-panner mono routes; and no new stereo/layered nodes in world, cues or propulsion. The existing held-note tests now check nominal-pitch body and layer-relative sustain, preserving the intent while accommodating the new patch.

`focused-tests.log` and `typecheck.log` contain raw final outputs. `execution-evidence.json` records actual scope/outcomes; `source-pins.json` pins tested working bytes and checks that Main/state/score/title melody/clock files exactly match the Root base after newline normalization. Formatting and diff whitespace checks passed. No broad build, browser, server, GPU/model work, recording asset or simulation/renderer edit occurred.

Remaining gates: Root's fresh native stereo title/Continue/mute renders with source hashes and peak/RMS/nonfinite checks; mono downmix and instrument/listening review; physical output-device comfort/performance; perceived reference-song match. Authored score notes/octaves/form and exact transcription remain provisional.
