# Actual browser audio capture review

All six declared desktop scenarios completed with native Web Audio, MediaRecorder and analyser output from the unchanged production AudioEngine. No measured DSP, recording, mute, recovery, voice-cap or branch-cleanup failure was found in these captures. Listening, physical-phone recovery and sustained natural gameplay remain Pending.

## Exact source and scope

Root operated the visible local QA page at `http://127.0.0.1:4188/` through the Codex in-app browser on Windows. The capture user agent identifies Chrome 155; the native context reports 48,000 Hz and base latency 0.01 seconds. All six reports observed one engine-context lifetime. These are explicitly declared charge/pulse requests through real `AudioEngine.play`, not natural Game shots, enemies or rewards.

- Original and byte-identical copied AudioEngine: `d74adf461c889a12ea1b36ad5888fc35121a79710a20bee8b53aaa70b67c94e8`.
- Compiled real AudioEngine: `b6ad9a8f86900b4f3ae20ba3ac69814d9022ba93a4861c53e4fbff79af80e0de`.
- Compiled probe actually served during the run: `a2fc99c92e96dfd30d1c2732823d093bda6c94284e16e93c43a40f8c2a75b4e0`.

The recording/analyser branches attach to the engine's existing final compressor. Speakers remain connected. There is no AudioContext replacement, microphone, offline-render fallback, dependency, gameplay mutation or production export. Strict installed TypeScript 5.9.3 compilation and eleven actual HTTP/source-pin preflight checks preceded browser operation. Tested served source and compiled bytes were unchanged throughout the captures.

## Measured native output

Values below are the native analyser's sampled peak and aggregate window RMS after the existing compressor. Windows use 2,048 real samples about thirty times per second. They are not true-peak, LUFS, loudness or hearing scores.

| Scenario / measured stage | Peak | Window RMS | Maximum effects voices |
|---|---:|---:|---:|
| Charge / cue | 0.0599248 | 0.00846080 | 1 |
| Charge / quiet tail | 0.00000112677 | 0.0000000604669 | 1 overall |
| Pulse / cue | 0.125000 | 0.0114637 | 1 |
| Pulse / quiet tail | 0 | 0 | 1 overall |
| Music + effects | 0.187885 | 0.0100420 | 1 |
| Before master mute | 0.125596 | 0.0102320 | 1 overall |
| Steady master muted | 0.000000539654 | 0.0000000259721 | 1 overall |
| After unmute | 0.124667 | 0.0107177 | 1 overall |
| Resumed | 0.124830 | 0.00977218 | 1 |
| Declared 100-cue burst | 0.757215 | 0.0515949 | 16 |
| Post-burst cleanup | 0.00000574557 | 0.000000467794 | 0 in every cleanup sample |

Charge tail RMS is 0.0007147% of its cue-stage RMS. Pulse's sampled tail is zero. A preceding muted music-bus transition leaves a small measured charge-baseline/tail residue, so this report does not substitute an exact-zero assertion for actual values. The steady muted RMS is 0.0002538% of the preceding stage and 0.0002423% of the later unmuted stage. Cues were actually requested during master mute; the bus kept their final output strongly attenuated.

Recovery recorded 24 suspended windows with null measurements, not fabricated silence. Their native context clock stayed exactly 86.712 seconds. The paused pulse request added zero effects voices; actual resume resolved running at wall 1,817.6 ms, and a later pulse added one voice with positive measured output. This proves this desktop suspend/resume interaction only.

The declared burst issued 100 real alternating requests, reached the unchanged sixteen-effects-voice ceiling, and returned to zero effects voices by the cleanup stage at wall 858.9 ms. Music never exceeded five voices across these captures, beneath its unchanged thirty-two cap. This does not establish that every notification remained perceptually distinct under stealing.

Every report is complete, uncancelled, failure-null, finite and within the probe's bounded sample/action/data limits. Both targeted branch disconnects succeeded with no errors; native recording tracks ended. Root then used the actual Dispose button. `artifacts/mote-audio-dispose-browser.txt` records visible status: original context closed, zero voices and zero engine graph nodes. No further engine was created.

## Recorded files and independent decoding

`capture-inspection.json` pins every report and original native audio file. `inspect-captures.py` reads those existing files without changing them, checks the declared scenarios/source/caps/lifecycle records, and uses installed ffprobe/ffmpeg to inspect and decode their real bytes. All six contain one 48 kHz Opus audio track with finite, nonzero decoded samples.

| Scenario | Original capture ID | Bytes | Decoded audio seconds |
|---|---|---:|---:|
| Charge | qa-1791666237851-3790f738 | 14,761 | 2.16 |
| Pulse | qa-1791666254186-d54a9f5a | 9,680 | 2.22 |
| Mix | qa-1791666267611-4088de80 | 97,065 | 6.00 |
| Mute | qa-1791666285020-bc901c07 | 55,126 | 4.68 |
| Recovery | qa-1791666322936-22112ad0 | 46,689 | 2.88 |
| Cap | qa-1791666369546-3dbbb477 | 27,162 | 2.28 |

The recovery recording's 2.88 decoded seconds differs from 3.708 wall seconds because the native audio clock paused. It must not be treated as a wall-aligned recording of synthetic suspended silence. Container encoding/delay also differs slightly from the declared wall stop in other captures. Whole decoded peak/RMS values are recorded separately; lossy Opus output is not numerically identical to live analyser windows. No audio was regenerated, normalized, resynthesized or auditioned by this reviewer.

## Preservation and remaining gates

`frozen-hashes.json` preserves authoring code, exact served binaries, capture reports/raw recordings, this review and the visible Dispose evidence. Dynamic service logs/PID are excluded. After root finished and closed the temporary tab, the owned node process 54088 was checked against this folder's exact `server.mjs` command line and stopped; port 4188 no longer has a listener. The probe is excluded from the shipped game.

Remaining acceptance is human listening/mix judgment, real Mote combat cancellation and crowded notification audibility, browser interruption/soak, and named physical mobile devices. The independently native-proven phase-entry/real-pulse Game event semantics are separate evidence; these declared browser cues do not retroactively become natural combat events. Static Scar appearance receives a separate pinned screenshot addendum under `work/next-slice-art-review/`.
