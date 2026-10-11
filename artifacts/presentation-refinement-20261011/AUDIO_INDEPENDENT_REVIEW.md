# Independent audio engine and reproducible helper review

No blocking correctness defect found in the inspected current engine or declared helper workflow. This is source review plus five focused Node checks. No server, browser, new audio context, microphone, native DSP render, listening session or release build was started by this review.

## Exact inputs

Observed root HEAD: `a73af3793ba938530925467726b8989b17dea619`. Tracked helper commit: `1a6f81f5843135de9bc0761b211537e896e7925c`. `source-pins.json` records 15 actual inputs, their retained raw snapshots, raw/LF-normalized hashes and committed blob hashes. All match the observed committed source after CRLF normalization; all nine helper/document files match the helper commit after the same normalization.

Current `audio.ts` has raw SHA256 `d0507d2aed1dbc51c2fb7513a57187e472ff16b09d508ddb671abd75b285d79f`; LF-normalized/committed SHA256 is `3356a60a48865b4a55d6152c5f2a5914a7447e15444b2bc53b3c4e58ac240576`. Thus the documented repaired run identifies the same substantive engine source. This review does not rerun or newly certify that historical PCM result.

## Engine findings

- Music dry and echo return, plus the effects bus, all reach `master → compressor → WaveShaper → destination`. The final curve is monotone, odd, finite, linear through the normal-level region and below 0.915. The helper observes the actual destination, including this ceiling; it does not capture the upstream master alone.
- The music feedback gain is 0.16; echo wet gain is 0.12. Engine sine/triangle/seeded-air sources share a lowpass and a separate duck stage. Important admitted cues and the low-hull alarm lower propulsion without repeatedly modulating the score bus.
- The engine bounds active music/effects to 32/16 voices. Persistent engine/alarm voices count against effects capacity and are excluded from transient stealing. Suspended/disposed gates, callback removal, voice/source disconnection and resume epochs are present. The death path retains one already-created death cue, gates the echo return and caps its timer at 800 ms; this offline helper deliberately does not exercise that realtime lifecycle.
- Main supplies fresh authoritative `Hud.audio` before resuming flight/recovery audio. The pure mixer clamps finite state, sanitizes champion identity and restricts new notes to two theme identities. These are observational audio changes, not fabricated combat or progression.

## Helper and measurement findings

- Preparation uses installed TypeScript 5.9.3, exact historical Git bytes and the current tracked mixer. Source/module/helper hashes are retained. Historical engine-only comparison with the current mixer is explicitly disclosed; it is not a historical whole-score replay. `transpileModule` is compilation, not a substitute for the repository's separate strict TypeScript check.
- The page runs four sequential, finite 48 kHz stereo fixtures. Its page-local constructor delegates node creation, graph routing, native time/sample rate and destination to the actual OfflineAudioContext. Only lifecycle/state and the 25 ms scheduling callback are adapted. Native suspension time is recorded rather than replaced with assumed timer time. Graph creation includes the real WaveShaper.
- PCM peak/RMS count both channels; nonfinite values are separately counted and fail the render. `over_unity` means absolute value at least one, not a claim of float clipping. Float32 WAV interleaving preserves over-unity samples. The FFT statistic uses eight Hann-windowed, channel-zero windows and excludes DC; it is spectral power, not loudness or a stereo-wide audibility assessment.
- A failed render/preservation visibly fails. The normal finally path disposes the engine and restores the three page globals. Successful cleanup asserts zero voices and captured scheduler callbacks. Download object URLs are revoked before the next run. Native graph failure during disposal is not an independently injected browser acceptance case.
- Local serving exposes only fixed helper/module/pin filenames. Captures require the exact configured Origin, a finite filename/run pattern, a 12 MiB maximum and exclusive `wx` creation. It never silently replaces a process on an occupied port. Default lifetime is 30 minutes with bounded signal cleanup. The runtime/public game does not import these development scripts.

One nonblocking local-tool constraint remains: output containment is lexical and the filesystem calls follow links. The CLI requires a path beneath `work/`, but it is not a sandbox for a hostile pre-existing junction/symlink tree; exported programmatic preparation also trusts its caller's output argument. Use a trusted fresh developer output directory, as the documented workflow does. There is no exposed arbitrary HTTP path or upload filename that chooses another output location.

## Executed focused checks

```text
node --test --test-name-pattern="all real buses reach one final bounded soft ceiling" scripts/test/audio.test.mjs
1 passed, 0 failed

node --test scripts/qa/audio-review/metrics.test.mjs
4 passed, 0 failed
```

Logs are retained here. The first is the actual engine with a recorded graph API; the other four are analytical metric/export checks. They are not native browser PCM or hardware evidence. The existing seven preparation/HTTP/cleanup tests were inspected but not repeated because this task prohibited starting servers/new clients. Root's already-recorded 11-test run remains separate evidence.

Human listening, natural combat balance, speaker/headphone comfort, realtime/mobile lifecycle and physical-device acceptance remain pending as documented. No Wiki source, release source, captures or tracked files were changed.
