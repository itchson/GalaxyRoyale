# Conservative source-driven percussion control

Actual work-only generated audio is ready for review. Production remains unchanged on clean `b27bcf823f169f32313c0a424d209ea3ddcf6f9f`. This candidate is a texture control, **not a recovered song or accepted percussion transcription**. All prior hook8/420/all443 sources and outputs remain sealed; preservation verification is in `preservation-and-source-pins.json`. Root assigned task `task-e418ca77`; actual running observation is `run-1508859a`, revision1127.

The original is the exact decoded42.68s stereo48k reference (`5c48d152…`, original MP3 `95fcb149…`). The fixed443 tonal control (`d1b05f46…`) retains all default note estimates and their upstream official timestamps. No raw contour bends, uniform drum grid, global timestamp offset, master normalization, limiter or source waveform/residual playback were added.

New review files in this directory:

- `fixed443-with-percussion.wav`: full42.68s, SHA `ee7f7dc69813887bb480d922aa4ccf8ffd57b5d86f57906596eb289d4894bed2`.
- `fixed443-with-percussion-first8.wav`: exact8s prefix, SHA `7962f7ba81a23d610d1fe7cc4f93a6b8cff7d3ad1c340751ce9941bf127f37ec`.
- `percussion-layer.wav` and `percussion-layer-first8.wav`: isolated generated transients.
- `first8-spectrogram-comparison.png`: actually rendered/viewed same-scale original/control/new PCM; no gameplay image or listening claim.
- `percussion-event-patches.json`: all407 measured candidate peaks, with explicit selected/omitted status, scalar envelopes, covariance and uncertain low-body parameters.
- `comparison-and-checks.json`, `render-report.json`, `budget-and-tempo-probe.json`, `low-band-gap-probe.json`: actual measured results and limits.

The source's previously frozen407 soft-percussive peaks include fast pitched attacks:203 are within30ms of estimated tonal starts. The renderer uses the same Hann2048/hop256 soft mask, then caps narrow spectral line peaks at a nine-bin frequency median. It subtracts earlier local band-power baseline, selecting bright transient excess (high-band share>=40%) rather than all peaks. Near estimated note starts, only the two upper bands may become noise; mid noise needs an independent bright event. These are conservative design thresholds, not accepted source instrument labels. Fifty selected high-noise events still overlap estimated tonal starts, so contamination remains possible.

Actual selection is99/407:92 noise events and nine uncertain low-body chirps (two overlap), only six selected events in first8. Scalar per-channel RMS envelopes are derived from source band-power excess; newly seeded/filtered noise supplies every output sample. Measured local spectral covariance controls an independent generated stereo pair (clamped±0.95). A falling low-band centroid plus measured decay enables a finite mono pitch-drop hypothesis; this does not prove a kick rather than a bass transition. No extra repeating glitch pattern is invented; short broadband events remain unreviewed glitch-like hypotheses.

All selected gates are finite<=135.333ms, maximum two simultaneous hits. The conservative source estimate is four transient carriers (two for stereo noise plus optional mono body). Joint443-note plus selected-hit data peak is18 carriers at8.88677s (16 tonal+2 transient), using two periodic-wave carriers per stereo note rather than an oscillator per harmonic. This estimate excludes future queues, crossfade, filters, effects and echo; it is not a WebAudio CPU/resource/mobile acceptance result. One two-channel generated noise buffer per hit could reduce node count, but that implementation has not run.

Actual mix measurements use unmodified levels, mono Welch4096/2048 normalized30–16k and separate stereo correlation:

| Window / mix | RMS | Peak | 2–6k power | 6–16k power | Stereo corr. |
|---|---:|---:|---:|---:|---:|
| First8 original | .090761 | .520773 | 10.707% | 3.676% | .735136 |
| First8 fixed443 | .075263 | .560120 | 3.914% | .138% | .773897 |
| First8 plus transients | .075295 | .560120 | 3.951% | .174% | .773590 |
| Full original | .099541 | .595332 | 12.563% | 4.742% | .728559 |
| Full fixed443 | .093907 | .850000 | 3.306% | .117% | .738597 |
| Full plus transients | .094037 | .850000 | 3.400% | .224% | .737841 |

The layer alone is peak.160723/RMS.004844. Combined output is finite/unclipped without changing the sealed tone gain. Its high-band change is small and remains far below the original; increasing noise gain to hit the aggregate band numbers would not recover sustained instruments. The actual source/control spectrogram also shows missing bass before1.5s. A separate0.65–1.45s30–180Hz zero-phase bandpass probe measures original monoRMS.069549 versus control.000001434; only one estimated overlapping note (MIDI83) exists. The original's strongest low Welch bin64.453Hz is descriptive, not an accepted fundamental/note. This is a concrete arrangement omission, not a percussion mix problem.

Next work should recover the missing source-supported bass and fit sustained harmonic families. Keep the conservative transient layer as an audition/control, review true source attacks individually, and replace the false uniform title drum pattern only after event review. Envelope centers are blurred by42.7ms Hann windows; the selected start search can move<=26.7ms earlier. The independent known-tone calibration validates retaining upstream note time conversion, not drum onset accuracy: no global offset or naive frame clock is substituted. Raw contours remain diagnostic because constant tones themselves showed about+33.3cent false estimates.

Production scheduler brief: preserve fractional source onset/gate offsets and the42.68s pickup/rest/end extent. The source flux autocorrelation has a167.910BPM quarter-lag candidate and a similarly strong112.5BPM alternative; subdivision ambiguity prevents declaring a beat grid here. Once a source beat unit is reviewed, map source seconds to fractional beats and call the existing shared clock's arbitrary `timeAtBeat` for each event cursor. Retain smooth tempo ramps/Continue phase, own loop-boundary events once, retime active envelope/gate knots without retriggering, and cancel/disconnect queued/active sources on title exit, mute or hidden/mobile interruption. Preserve note phase/sample-origin conventions or refit deliberately. Do not force512 integer steps/32bars or feed recovered melody over guessed `SCORE_CHORDS`.

Actual checks: all407 records retained; gates/envelopes finite, ordered and bounded; no mid noise within30ms of estimated note starts; output zero outside gate union; combined-minus-tone equals generated layer within2.48e-8 Float32 rounding; both8s prefixes exact; all four generated WAVs byte-identical on a second actual render. First candidate-feature schema failure (`KeyError:0`) and source copy/retry remain archived. Two rejected Studio timestamp submissions are also preserved before the actual accepted running record. No new model inference, runtime edit, normal build, listener/browser, native PCM, ear/fidelity/human or GPU acceptance is claimed.
