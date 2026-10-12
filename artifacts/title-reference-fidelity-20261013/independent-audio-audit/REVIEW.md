# Title reference: source, synthesis and timing review

Reviewed 2026-10-13 by engine_fx. Work only; no production edits, native build, browser audio execution, uploads or listeners. The tracked worktree remains at `4ab10c0a7018501ebfaa7dd978101ba0d979f6fb`. This is a source/math review and one actual generated-tone inference experiment, not hearing or fidelity acceptance.

The frozen eight-second additive experiment has a genuine source-derived periodic synthesis path. Its candidate accumulator does not replay source PCM. The model's raw bend output is not trustworthy enough to drive synthesis blindly: our actual constant-frequency calibration would detune steady tones by about 33 cents. The existing integer-step score API also cannot express the experiment's unquantized note onsets without a scheduling change.

## Exact evidence scope

The author freeze is `C:/Users/mitch/.codex/worktrees/galaxy-audio-refinement/galaxy-royale/work/title-reference-fidelity-experiment/frozen-hook8/MANIFEST.json`, SHA-256 `eb41e7ab2976dd0bb348a889a7c099b83feb71ce0856fb74b8ec258f67df05a6`. All 40 declared frozen files were independently read and verified. `hook8-numerical-audit.json` contains the actual independent coefficient/PCM checks, with its exit-zero log retained. The new calibration is wholly under this review's `calibration/` directory and has separate generated-input/model/API/output provenance.

The exact user MP3 is `C:/Users/mitch/Downloads/Galaxy Royale (3).mp3`: 1,032,175 bytes, SHA-256 `95fcb149bb55da752c4d93c18fcc1913a23b70ccafecb52d8db5548dc64755d6`. The frozen decoded mono signed-16 WAV is 941,094 samples at 22,050 Hz (42.68 s), SHA `c1095d741fabb279f33fdb0216d3a10521b30b129903daa99eb15bb8bd5a3d02`. The stereo float32 WAV has 2,048,640 frames at 48,000 Hz (42.68 s), SHA `5c48d1528353c3ab1b6d3d9973f3c4e152091e97b48bc459a7d3436e66f78900`. The separate labeled eight-second reference WAV is an exact float32 copy of that stereo prefix; it is not the synthesized candidate.

Author Python 0.4.0 attempts failed before model execution: first `pkg_resources` was missing; the retained second traceback reports WinError4551/Application Control blocking the llvmlite load. These are not successful model runs. The official JS CPU path actually ran. Our calibration uses the same pinned `@spotify/basic-pitch` 1.0.1 API, package lock and model, with installed API bytes checked against the freeze and exact frozen JSON/weight bytes loaded into TFJS. No policy changes are part of this work.

## Model clock and bends

Primary source is the frozen official `cjs/inference.js` and `cjs/toMidi.js`, rather than an assumed frame rate. Input is mono22,050 Hz. FFT hop is 256 samples, annotation FPS is floor(22,050/256)=86, model input window is 43,844 samples, overlap is 30 frames, left padding is 3,840 samples, and input-window hop is 36,164 samples. Unwrapping trims 15 overlapping frames from each side and trims final output length using the original input sample count. `Float32Array` is a supported input; our experiment uses it directly.

The official exported note clock is:

```
t(frame) = frame * 256 / 22050 - 0.010326077097505668 * floor(frame / 172)
```

This preserves upstream conversion, including its window correction. The numerical conversion of a final frame index alone is not a measured source-clock offset. Our generated gates below are the actual onset calibration. No fitted global correction was applied to the song or runtime.

Contours have 3 bins per semitone. `addPitchBendsToNoteEvents` selects a Gaussian-weighted contour maximum and exports the integer displacement. Interpreting an accepted displacement therefore requires bin/3 semitones (100*bin/3 cents), but correct units do not establish accuracy. The installed upstream MIDI exporter puts these raw integers in a single track's pitch-bend events; it does not perform a per-note bin-to-semitone/RPN-range conversion. Its MIDI file remains a diagnostic, not a suitable polyphonic playback contract. Retain exact raw frame keys for bend timing rather than assuming that uniformly spreading contour frames over note duration is equivalent across window boundaries.

## Actual known-onset calibration

`calibration/run-calibration.mjs` generated 43 s of mono float32 PCM at 22,050 Hz with 14 isolated, gated 440/880 Hz sinusoids, alternating pitches69/81. Gates use sample-rounded ground truth, peak0.6, linear4 ms attack/12 ms release, and positions before/after early and late 36,164-sample window boundaries, plus a final gate at42.35 s. No original-song samples participate. The original generated WAV, ground truth, model arrays, raw note frames and converted notes are retained.

Actual official TFJS CPU inference and decode completed with exit0. It produced 3,698 rows of frames/onsets (88 pitches) and contours (264 bins). Both decoder settings recovered all 14 exact pitches with no unmatched/missing candidates in this isolated fixture: default(.5/.3/min11) and fast(.4/.25/min5). The declared matching rule is unique same-pitch nearest onset within0.75 s, with raw and converted event counts both14.

| Measurement | Default | Fast |
| --- | ---: | ---: |
| Converted onset minimum / maximum | -27.913 / +1.556 ms | same |
| Converted onset median | -9.335 ms | same |
| Error slope versus source onset | -0.1313 ms/s | same |
| Naive uncorrected frame error slope | +5.0573 ms/s | same |
| End error minimum / maximum | -44.784 / +11.125 ms | -44.784 / +22.735 ms |
| End error median | +5.090 ms | +16.700 ms |

The near-input-tail end is 44.784 ms early. The known fixture supports retaining the official correction instead of a naive hop clock. It does not establish exact timings of the mixed song's notes, drums or final cadence, or justify a universal -9 ms adjustment.

`constant-tone-bend-counterexample.json` quantifies the important failure: default retains 525 bend frames, of which 522 are +1 bin and only3 are0; fast retains536, of which533 are+1 and3 are0. Every known steady note has median proposed +33.333 cents despite a true bend of0. Thus 99.43%/99.44% of these retained frames would be wrongly detuned if passed directly to synthesis. This is a blocker to blind raw-bend use. It does not justify a global -1 bin correction or infer the song's true expressive bends. Fixed-pitch and reviewed-bend controls must remain distinct.

## Eight-second additive path

The frozen author `reconstruct_hook.py` retains all71 model events in the raw JSON and builds69 periodic note patches after explicitly excluding2 sub-C2 estimates. Each patch fits a Hann-windowed central segment of the mixed stereo reference, at most100 ms, with:

```
C_h = 2 * sum(x(t) * w(t) * exp(-i*2*pi*h*f*t)) / sum(w(t))
y(t) += Re(C_h * exp(+i*2*pi*h*f*t)) * authoredEnvelope(t)
```

Both coefficients are stereo. Candidate synthesis uses only these complex coefficients, a periodic carrier and an authored4 ms attack / bounded18 ms release. Reading source PCM to estimate coefficients is source-derived analysis; it is not waveform/sample playback. The labeled original reference export is a separate path. The candidate uses fixed equal-tempered pitch and does not apply raw contour arrays.

We independently recomputed nine coefficient checks from the first three patches at harmonic1,2,last. All had max absolute complex-coefficient error0.0. All69 patches use at most20 harmonics; maximum partial is15,840 Hz, below the declared16 kHz ceiling at48 kHz. Candidate contains no nonfinite samples. RMS is0.0907613707347 versus reference0.0907613707448 because the author deliberately aligned it with one static gain; peak is0.679812. Stereo correlations are0.77337(candidate) versus0.73514(reference). These numbers establish the declared computation and bounded output, not musical resemblance, correct instruments, stems or notes.

The mixed-track projection can copy neighboring notes' harmonics into a patch, double-count octave/harmonic aliases, and miss dynamic envelopes, transients, unpitched percussion and effects. It is a scalar harmonic projection, not joint isolated-voice fitting. Bass exclusions must remain explicit; whole-song completeness requires examining every retained low event rather than silently dropping it. The later all443 freeze was subsequently inspected in the separate `ADDENDUM-FULL443.md`; none of the original prototypes were rerendered or musically accepted by this review.

## Exact runtime timing/phase contract

The frozen score `titleNotesAtStep()` uses integer equality and a512-step loop at169.275 BPM, a45.36996 s period. The reference is42.68 s. Fractional onsets inserted into this current API would never match an integer scheduler step. This is a source-defined limitation of the existing authored score, not a claim of accepted transcription.

The smallest integration must retain each accepted event's unquantized onset and duration, plus a defined loop period. It can bucket events by floor(step) and pass their fractional offset to `MusicClock.timeAtBeat`, or use a sorted continuous event cursor. Each event needs one scheduled birth per loop; lookahead cancellation, tempo retargeting, title/world transition, interruption/resume and late-frame skipping must reschedule future notes without duplicate or missed births. Use event duration/end coordinates rather than a rounded `steps` count. The existing0.18 s lookahead and16-step work cap must also bound event scheduling and oscillator/node counts: 32 logical music voices is not a bound on20-partial stereo oscillator nodes.

The coefficient phase basis must be explicit. For this fixed-frequency absolute-song-time prototype, a browser oscillator beginning at local source onset `s` needs `D_h = C_h * exp(+i*2*pi*h*f*s)`, then real=Re(D_h), imag=-Im(D_h). Use local song time, not large absolute AudioContext time. A coefficient already fitted in note-local integrated phase must not receive that rotation again. With variable pitch, integrate the frequency to obtain phase; `frequency(t)*t` is not that integral. Cull harmonics using maximum accepted bent frequency and actual AudioContext sample rate.

The WebAudio Fourier convention uses real cosine coefficients and imaginary sine coefficients; explicit `disableNormalization:true` is needed if fitted amplitude is to survive the default normalization. This mapping follows the [W3C WebAudio waveform definition](https://www.w3.org/TR/2021/REC-webaudio-20210617/#waveform-generation). Bounded stereo PeriodicWave voices are one practical alternative to40 oscillators per note, but browser phase/amplitude output, transitions, loop seams, CPU budgets and human comparison still require actual execution. No such implementation or acceptance is claimed here.

## Handoff decision and open gates

Source-derived fixed-pitch patch/data experiments may proceed as separately labeled controls. Do not promote raw bends, RMS alignment, approximate transcript counts or generated-tone timing to true-song fidelity. The calibrated clock, zero-bend counterexample and phase basis should inform the next bounded continuous-note adapter. Remaining gates are cleaned source notes/voice assignment/percussion, complete bass and final cadence, accepted bend handling, real WebAudio render/clock/normalization checks, bounded production voices/cache, transition/loop tests and human same-position listening. No real-song timing correction or production changes are made by this review.

All manifest pins in this review describe raw on-disk bytes. They are not canonical Git-blob claims. The generated calibration is distinct from the author freeze and from the user's reference audio. Original failures and outputs remain unchanged.
