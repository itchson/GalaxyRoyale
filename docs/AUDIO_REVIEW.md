# Native audio DSP review

The development-only helper in `scripts/qa/audio-review/` renders the actual current `AudioEngine` through the browser's native `OfflineAudioContext`. It is not imported by the game, Vite, the service worker or the public build. No microphone, external audio, model, new dependency or fabricated PCM is used.

From the repository root, with Node and the existing locked dependencies:

```sh
pnpm install --frozen-lockfile
node --test scripts/qa/audio-review/metrics.test.mjs scripts/qa/audio-review/helper.test.mjs
node scripts/qa/audio-review/prepare.mjs
node scripts/qa/audio-review/server.mjs --port 4188 --autoquit-seconds 1800
```

Open `http://127.0.0.1:4188/` and click **Run native DSP review**. Nothing creates an audio context or renders before that action. Use `Ctrl+C` to stop the server; SIGINT/SIGTERM perform bounded cleanup. It exits automatically after 30 minutes by default; `--autoquit-seconds 0` disables that timer. An occupied port is a visible error: the helper never kills another process or silently chooses another port. `--port` accepts an explicit alternative. No server is automatically launched by preparation or tests outside their short-lived loopback fixtures.

Preparation compiles real tracked `apps/game/src/audio.ts` and `audio-state.ts` with the existing pinned TypeScript **5.9.3**. It obtains the historical engine directly from Git:

```text
3bf0c02b5e9fc0011c0ce6921bfaf0fbe664c65d:apps/game/src/audio.ts
SHA256 6e06ef04f40367ab8519adfd445258b0c284d6b1c7ac3d955ea212962c629f64
```

A clone must contain that history. If a shallow clone lacks it, fetch its history (`git fetch --unshallow`) before preparation; there is no ignored snapshot fallback. The historical engine imports the **current shared mixer**, matching the controlled engine-only baseline comparison. This is not a replay of an entire historical score/state implementation.

Generated modules, source/module/helper SHA256 pins and captures go under ignored `work/audio-review/`. To keep an active service or existing captures untouched, both commands accept the same separate output directory:

```sh
node scripts/qa/audio-review/prepare.mjs --out work/audio-review-fresh
node scripts/qa/audio-review/server.mjs --out work/audio-review-fresh --port 4190
```

The output must be inside this checkout's `work/`. Stop its server before preparing again. At startup the server verifies compiled module and tracked helper hashes. Static GET paths are allowlisted; capture POSTs require the exact localhost Origin, bounded run/filename patterns and at most 12 MiB. A report or WAV is written once (`wx`) and cannot overwrite a previous capture. The page exposes report/float-WAV download links as well. A failed native render or failed preservation is visibly reported; it does not substitute samples.

## Declared fixtures and measurements

Four sequential stereo, 48 kHz native renders exercise the real sequencer and voice limits:

1. Six seconds of historical engine at master/effects `.75`, music `0`, speed/throttle `1` for four seconds followed by two seconds release.
2. The current warm engine with exactly the same settings and state.
3. Eleven seconds of eight sequential champion themes, combat `.8`, hull `.08`, throttle `.9`, speed `.7`, settings `.75/.4/.75` and 34 declared authored cue calls.
4. Five seconds at maximum buses with score/engine/alarm and 16 explosion calls at one second, 16 death calls at two seconds and 16 upgrade calls at three seconds. The actual cap/stealing applies. These are deliberate stress calls, not natural gameplay or the death-screen lifecycle.

The isolated page temporarily substitutes a constructor that delegates every graph/node operation, destination, time and sample rate to a real native `OfflineAudioContext`. It exposes running state and no-op engine lifecycle methods only to allow offline scheduling. Captured 25 ms scheduler callbacks advance after **actual native suspend points**, using quantum-rounded native time; rendering resumes through the native context. Overrides restore in `finally`. This adapter cannot prove realtime/mobile suspend or recovery behavior and never accesses the game's QA API.

Reports measure raw PCM peak/RMS, sample count, `over_unity` and nonfinite samples. `over_unity` counts `abs(sample) >= 1`: this flags potential clipping at a downstream output but does **not** claim the float buffer was clipped. No metrics or WAV samples are clamped. IEEE float32 WAV preserves over-unity samples. Eight Hann-windowed 4096-sample FFT windows measure the fraction of engine power above 1 kHz; this is not perceived or A-weighted loudness. Source pins, native suspend times, cue dispatch times, actual voice maxima and cleanup counters accompany the measurements.

## Recorded result and limits

An actual visible Chromium run of the original authored helper before the final ceiling (`qa-1791686360305-ceff9789`) found a max-bus peak **1.0465022**, with **six samples at or beyond unity out of 480,000 channel samples**. That immutable failing capture justified the output repair; the compressor alone was not a brick-wall guarantee.

The next actual run (`qa-1791686562452-e304e589`) used current audio source SHA256 `3356a60a48865b4a55d6152c5f2a5914a7447e15444b2bc53b3c4e58ac240576`. All four declared fixtures had zero over-unity/nonfinite samples. The max-bus peak was **0.91484094**, representative mix peak **0.29541355**, effects reached but did not exceed **16 voices**, and cleanup reported zero voices/scheduler callbacks. Engine steady RMS was **7.43695 dB lower** than the prior engine; power above 1 kHz fell from **0.0448868** to **0.000427895**. Independent float-WAV decoding verified all eight original captures, totaling **5,376,000 channel samples**, against their raw sample metrics and hashes. Those archived reports used the earlier field name `clipped` for the same at-or-beyond-unity count; they remain unchanged.

This tracked helper preserves that rendering algorithm and declared fixtures, with reproducible Git baseline preparation, stricter serving/pinning, clearer metric naming and bounded cleanup. The recorded result belongs to the source and original run above; it is not a claim that the newly tracked page has been clicked again or that future source changes automatically pass. Run it again when changing the mix.

Helper Node tests validate real source compilation/hash checks, HTTP path/Origin/size/immutable-write behavior, occupied-port and autoquit cleanup, and metric math/float export. Analytical tone fixtures are explicitly math tests, not browser DSP evidence. Recorded graph tests in `scripts/test/audio.test.mjs` cover bounded voices and lifecycle separately. Human listening, natural combat mix, speaker/headphone comfort, realtime/device performance, mobile recovery and physical-device acceptance remain pending.
