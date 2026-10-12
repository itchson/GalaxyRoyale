# Independent fuller stereo title source review

Reviewed integrated audio candidate `a1c16b68c5312822ae4efc214036825ac6471d8f` and frozen testing Root `f4b8882b568339f0aa164c6aaf99a174b59cc105`. The author candidate `6f24d837` was based on Root `d028f0d7a3c589b5896f536f08e76aa504cb2923`. This reviewer wrote only `work/title-width-independent-review/**`; the earlier reviews were preserved.

**No concrete blocking runtime source defect found in this bounded pass.** The layered title components retain finite gates, common stop ownership and the existing voice budget. Stereo and missing-panner routing are covered by actual recorded-API tests. Native sample quality, mono/downmix energy, perceived fullness, physical listening and reference fidelity remain separate acceptance work.

## Actual independent execution

On Node `v24.18.0`, both commands exited zero:

```text
node --experimental-transform-types --test scripts/test/music-clock.test.mjs scripts/test/title-theme.test.mjs scripts/test/audio-state.test.mjs scripts/test/audio.test.mjs
58 passed, 0 failed

node --experimental-transform-types --test work/title-width-independent-review/edge-review.test.mjs
3 passed, 0 failed
```

Raw output is retained in `focused-tests.log` and `edge-review.log`. No failure occurred in this independent pass. The current recorded harness includes StereoPanner creation support and is copied beneath this review's owned directory. No build, server, browser, reference audition or new native render was run by this reviewer.

## Source, envelopes and resource ownership

- A lead uses exactly three sources per authored note: filtered square pulses at +3 and −4 cents with opposite moderate pans, and a centered triangle body at nominal pitch. The held attack/release envelope remains finite. A pad uses eight sources: four nominal triangles and four filtered saw layers detuned by ±4 cents in the same register. The pad's chord slice limits it to four tones. Bass uses three centered components at the authored fundamental. Arpeggio and bubble add gentle static alternating pans.
- Source envelopes precede panning, and every panner feeds its existing voice gain. Filters, panners, source envelopes and sources are owned by that voice. The shared graph, buses, compressor, final ceiling, one-context transport and 32/16 voice caps are unchanged. Default `pan=0` creates no panner. World, SFX and propulsion stay on their prior routes and source counts; the focused regression checks this using actual world/cue/engine calls.
- Sustained inner pad envelopes and outer finite gates are unchanged from the earlier mix refinement. Retiming preserves the outer gate's evaluated current value, replaces the fade deadline and replaces all eight source stops. The independent Continue check during the pad attack moved all stops from `1.3991980505` to `1.4514851242` seconds. It verified the panners were still connected after the old deadline and every owned node disconnected once at the new deadline. The existing release/recovery retime tests, late terminal fade tests, future cancellation, asynchronous resume and cleanup tests also pass with the larger patch.
- The current authored data gives at most one lead per onset. Its compound backbeat bound is nine sources: lead3 + arp1 + bass3 + kick1 + snare1. This is a bound derived from the fixed current score, rather than a separate runtime source-admission counter. A future data change introducing simultaneous lead events would need the source bound revisited. With the present score and 32 music voices the conservative owned music-source bound is 288, while actual active counts are much lower.
- The 30-second focused rapid-state fixture reproduced maximum 21 music voices, 12 future voices, 71 owned music sources and 210 owned music nodes. Independent complete-loop runs over 45.669960 seconds exercised all 512 steps and the return into the next loop. Both stereo and mono fallback measured maximum seven music voices and 40 owned sources, with a maximum nine sources in one voice. Stereo measured maximum 129 owned nodes and eight panners in a voice; mono fallback measured 106 nodes and zero panners. Future onsets stayed inside 0.18 seconds, all stop deadlines stayed finite, and hard suspension released/disconnected the final nodes. These are sampled active API-resource observations, not CPU, heap or all-day browser measurements.

## Mono, phase and QA integration limits

- When native `createStereoPanner` is absent, production `attach` connects each envelope directly to its voice gain. Full-loop recorded fallback testing verifies one finite output path per layer, source bounds and cleanup. The scalar envelopes are retained; the direct mono sum removes equal-power panning's per-channel attenuation. Equal bus settings therefore do not promise equal perceived level between this fallback and the stereo route. Native fallback/downmix peak, RMS and listening comparison remain open.
- Pans remain within ±0.32. The patch adds no channel inversion, channel delay or modulation source; the centered bass also avoids stereo sub-bass separation. Detuned same-register oscillators naturally change their relative phase and beating over a note. Their resulting spectral/phase interaction and waveform energy require native measurements and listening; graph/envelope tests cannot certify mono comfort, absence of cancellation, audible width or improved fullness. The bass's scalar component allocation remains 0.095, which does not imply equal native RMS or waveform peaks across its new shapes and attacks.
- Root's current `scripts/qa/audio-review/review.js` creation-method set includes `createStereoPanner`. Its wrapper calls `target[name](...args)` on the real OfflineAudioContext and registers the returned panner with the graph nodes, satisfying the candidate's native factory-binding requirement. The helper is separately pinned at frozen Root because it is Root integration work.
- A limited QA compatibility caveat remains: the Proxy returns a wrapper for every name in its creation-method set without first checking whether the real native method exists. In an OfflineAudioContext implementation lacking StereoPanner, the production feature test would see that wrapper as a function and the wrapper would fail on invocation. The shipped engine's direct-context fallback is tested and unaffected. This is not a blocker for Root's current StereoPanner-capable native render, but that QA adapter would need to preserve native method absence before it could verify an unsupported-browser fallback.

## Provenance and pins

`review-pins.json` pins exact Root working bytes separately from canonical Git blob bytes/hashes, records the commands and actual outcomes, checks candidate/current source equality after newline normalization, and compares unchanged modules against the author base. The author audio/test pins match canonical Git/LF bytes; Root's CRLF working byte hashes are explicitly different. Frozen author handoff and logs are checked without treating normalized text equality as raw equality. The typecheck log's original mixed newline form is separately reconstructed to its pinned hash.

The author raw focused log reports 58 passes and the resource measurements reproduced above. The typecheck evidence records command/exit metadata; this reviewer does not claim a separate TypeScript run. Main, state, title notes, score/harmony and music clock are unchanged by the instrumentation candidate. Authored note data remains reference-inspired and provisional. No prior or concurrent native capture, supplied listening reaction or source-analysis result is promoted to new DSP/listening/reference acceptance by this report.
