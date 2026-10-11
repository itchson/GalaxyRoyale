# Warm thrust: actual DSP finding and output-ceiling repair

This follow-up supersedes the earlier source-only warm-thrust freeze for current production audio. Earlier handoffs and the first failed capture remain immutable. Owned production files are still only `apps/game/src/audio.ts` and `scripts/test/audio.test.mjs`. No game state, score registry, user volume settings, renderer, package or manifest was edited.

## Actual finding and repair

Root clicked the visible isolated native OfflineAudioContext page twice using CUA. First run `qa-1791686360305-ceff9789` measured the calmer engine and exposed compressor attack overshoot in the declared maximum-bus stress fixture: peak **1.046502233**, with **six samples ≥1 among 480,000 channel samples**. This is retained as failed native-DSP evidence, not relabeled as acceptance.

The production graph now routes every bus through **master → existing compressor → one native WaveShaper → destination**. The 4,097-entry float32 curve is linear for |x|≤.78. Above that knee it evaluates `sign(x) × (.78 + .15 × tanh((abs(x) − .78) / .15))`; its finite endpoints are approximately ±.914841. Native WaveShaper saturation outside the input range bounds large compressor overshoots at those endpoints. Oversampling is `none`, leaving quiet mix samples linear without modifying the user's gain settings. It adds one static node and approximately 16 KiB once per context, no voice, timer or per-frame allocation.

The second actual run `qa-1791686562452-e304e589` measured the repaired maximum-bus fixture at **.914840937 peak, zero clipped samples and zero nonfinite samples**. The representative mix peak remained **.295413554**. The same existing 16-effects/32-music caps and cleanup paths remained authoritative. No production-source change followed that passing DSP run.

## Native PCM results

| Declared fixture, repaired run | Peak | Whole-fixture RMS | ≥1 / nonfinite | Actual max music/effects voices |
|---|---:|---:|---:|---:|
| Accepted prior engine, default buses | .064656653 | .026843521 | 0 / 0 | 7 / 1 |
| Warm engine, default buses | .028138995 | .011362013 | 0 / 0 | 7 / 1 |
| Eight-theme representative score/cues | .295413554 | .037014597 | 0 / 0 | 12 / 3 |
| Maximum-bus capped burst | .914840937 | .115124777 | 0 / 0 | 11 / 16 |

The engine's **steady-window RMS is −7.436947 dB** relative to the accepted prior source. Its measured >1 kHz Hann-FFT power fraction changed from **.044886800 to .000427895**. These are native rendered-sample metrics, not subjective listening scores. The representative case is an authored state/event fixture, not natural combat. The adversarial burst intentionally calls 16 explosion, death and upgrade events at separate checkpoints; it does not infer game rewards or invoke a gameplay death state.

The adapter creates real native OfflineAudioContexts, delegates graph/currentTime/sampleRate, and advances captured scheduler callbacks only at actual native suspension checkpoints. Its running/no-op-resume facade exists solely for offline scheduling. It cannot prove realtime AudioContext/mobile recovery. All overrides restore in finally, and all cases report zero live voices and scheduler callbacks after disposal. The historical baseline remains the exact source `6e06ef04f40367ab8519adfd445258b0c284d6b1c7ac3d955ea212962c629f64`; the pre-ceiling warm source/module/test/pins are separately retained in `pre-ceiling/`.

## Executed verification and exact evidence

**31/31 focused checks pass**: 26 instantiate the real AudioEngine against a deterministic recorded Web Audio API; five exercise the unchanged pure state mixer. The added ceiling test covers all-bus routing, one static node, finite/odd/monotonic bounds, linear quiet samples, cap/stealing, node reuse on resume and one-time disposal. `npm run typecheck` passes. Four work-only PCM/FFT/export math tests had passed before actual rendering; they are explicitly mathematical fixtures rather than browser audio proof.

Eight actual preserved float32 WAVs from both runs were independently decoded with Python's float32 array, rederiving **5,376,000 actual samples** of peak/RMS/clipped/nonfinite counts and verifying each WAV's SHA256 against the native report. `raw-pcm-verification.json` records this. Native ffprobe independently confirms eight stereo 48 kHz `pcm_f32le` WAV formats/durations in `ffprobe-verification.json`. Export does not clamp samples or convert to int16, so the original over-unity values remain intact. Both reports and all eight raw WAVs are pinned in the new manifest.

First failed burst WAV: `22908da692f20c11553e6856373cb8b6285f71e8b46374f4b6346606ecc6c86a`.

Repaired burst WAV: `62bd667d3829a806609e58bde56e8f71887c66b10fd45184c678492938e57842`.

The dev-only page is under `work/audio-review/`; no public/dist build or publication was performed by this specialist. Root owns normal game packaging and integrated browser tests. The final source/harness/report pins are in `frozen-hashes.json`; `verify.py` reads pins and the two exact production snapshots without modifying anything.

## Limits

The observed native offline PCM establishes the stated fixtures and this desktop browser implementation. Human listening, perceived pleasantness, natural-game score balance, speaker/headphone suitability, mobile recovery, realtime performance and physical-device acceptance remain **Pending**. Digital sample limits are not a measurement of an analogue reconstruction or a physical sound system. The original failed run remains visible alongside the repair; previous source-only scalar estimates are not substituted for this measured DSP evidence.
