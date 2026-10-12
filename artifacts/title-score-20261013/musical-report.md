# Galaxy Royale title reference: analysis and native score proposal

This pass analyzed the supplied file and designed an engine score. It did **not** recreate or accept the song. No direct listening occurred in this agent's tooling; detected pitches, chords, meter and instrument roles remain analysis candidates. Spectrum/onset agreement is not proof of a perceived match.

## Source and reproducible evidence

- Source: `C:/Users/mitch/Downloads/Galaxy Royale (3).mp3`, 1,032,175 bytes.
- Source SHA256: `95fcb149bb55da752c4d93c18fcc1913a23b70ccafecb52d8db5548dc64755d6`.
- ffprobe: 42.680 seconds; audio stream 0, MP3, 48,000 Hz stereo, about 189 kb/s. Stream 1 is a 360×360 attached MJPEG cover, not another audio track. Container audio start is 0.023021 seconds; the times below refer to decoded PCM starting at zero.
- Tags name the title “Galaxy Royale,” artist “itchson,” and describe instrumental 8-bit arpeggios, bubble-slop, glitch-pops and a loop pickup. Tags are authoring hints, not verified audible instrument recognition.
- Local analysis-only WAV: `reference-analysis-mono.wav`, 22,050 Hz mono signed 16-bit PCM, 941,094 samples / exactly 42.680 seconds. SHA256: `c1095d741fabb279f33fdb0216d3a10521b30b129903daa99eb15bb8bd5a3d02`.
- The source MP3 was read and left untouched. No source audio was placed in shipped assets.
- `measured-analysis.json` includes raw ffprobe metadata, analysis script and WAV hashes, all tempo results, 2-second chroma/chord windows, and 125 ms upper-pitch candidates. `onsets.csv` is the measured transient series. `refined-timing.json` contains an independent grid-fit pass. `source-pins.json` pins inspected production/review inputs and analysis outputs.
- Runtime: existing Python 3.12.10, NumPy 2.5.2, SciPy 1.18.1, Matplotlib 3.11.1. No packages, models or GPU services were installed or downloaded.

Executed commands:

```powershell
ffprobe -v error -show_format -show_streams -of json 'C:\Users\mitch\Downloads\Galaxy Royale (3).mp3'
ffmpeg -hide_banner -loglevel error -i 'C:\Users\mitch\Downloads\Galaxy Royale (3).mp3' -map 0:a:0 -vn -ac 1 -ar 22050 -c:a pcm_s16le 'work/title-score-20261013/reference-analysis-mono.wav'
python 'work/title-score-20261013/analyze_reference.py'
python 'work/title-score-20261013/refine_grid.py'
```

Both Python scripts completed with exit code 0. Their saved stdout is in `analysis-run.log` and `refinement-run.log`. The analysis chart was visually inspected. No browser, game, audio output device, WebAudio score render, production build or studio acceptance was exercised by this pass.

## Measured rhythm and tonal evidence

Transient analysis uses a 1,024-sample Hann STFT / 128-sample hop: about 46.4 ms windows and 5.805 ms time steps. Positive log-magnitude spectral flux is separated into bass, mid and high bands. The earlier long-window transient trial was replaced by this shorter analysis; its numbers are not the final evidence.

| Measurement | Result | Interpretation |
| --- | --- | --- |
| Combined onset ACF, first tempo peak | 169.442 BPM, normalized ACF 0.51007, lag 61 | A fast pulse around 169–170 BPM is the leading candidate. |
| Combined onset ACF, next peak | 84.721 BPM, ACF 0.44982, lag 122 | Ordinary half-time ambiguity; this does not prove which pulse is the quarter note. |
| Bass onset ACF, whole clip | 169.442 BPM, ACF 0.29854 | A second band supports the fast pulse. |
| Bass constant-tempo comb fit | 169.275 BPM, phase 0.25255 s, mean sampled flux 0.06120 | Useful candidate grid, not a confirmed musical downbeat. |
| Combined/mid eighth- and sixteenth-pulse periodograms | About 169.2 BPM after dividing by 2/4 | The most stable periodic energy is in subdivisions. A quarter-pulse-only periodogram has other peaks and is weaker; retain the meter/downbeat uncertainty. |
| Bass ACF 0–10 / 10–20 / 20–30 / 30–42.65 s | 166.709 / 169.442 / 169.442 / 166.709 BPM | Finite hop resolution and different patterns move the local maximum. These results alone do not establish real tempo drift. |
| Global chroma profile | B minor 0.8135, then B major 0.5929 and E minor 0.5859 | B minor is the strongest tonal-center candidate. Correlation values are not probabilities. |
| Most prominent global pitch classes | B 0.1435, F# 0.1022, A 0.0973, G 0.0945, E 0.0938, D 0.0844 | Supports a B-minor family, with B acting as a common tonal anchor. |

Confidence: **moderate** for a ~169–170 BPM pulse family, **moderate** for B minor as the overall tonal center, **low** for a definitive meter/downbeat and precise chord progression, and **low** for octave-specific melody transcription. The requested combat tempo is **166 BPM** regardless of this source estimate; exploration remains **120 BPM**. “BPS” is interpreted as BPM.

Harmonic analysis uses an 8,192-sample Hann STFT / 256-sample hop, median harmonic/percussive separation, local spectral whitening and pitch-class folding. A 371.5 ms window helps frequency resolution but blurs rapid note boundaries. Upper-pitch candidates sum six harmonic partials and are particularly vulnerable to octave and chord-tone aliases. Instrument, voice, exact note length and note velocity cannot be read directly from those rankings.

## Time regions and hook candidates to audition

These regions locate evidence; they are not accepted verse/chorus labels.

| Decoded time | Actual analysis evidence | Composition question |
| --- | --- | --- |
| 0.00–2.25 s | Opening F#5/F#6 candidates give way to persistent B4/B5/B6 candidates; chroma often favors B/F#/E. | Which octave is the foreground voice, and which are harmonics or arpeggio accompaniment? |
| 2.25–4.00 s | F#4/5 and A4/5 candidates around 2.25–2.625; D5/D6 rises around 2.625–3.0. B minor and D major are near-tied chord candidates for the 2–4 s window. | Test a B-centered opening followed by F#–A–D; do not label this an exact hook until heard. |
| 4.00–8.00 s | B/D/E/G upper candidates, with G-major and E-minor chroma share close. | Establish bass roots before choosing G versus Em or a suspended voicing. |
| 8.00–10.00 s | E/A/B-related suspended candidates; B-related low-pitch salience. | Preserve the phrase's suspended color rather than force every note into a triad. |
| 10.00–12.00 s | D#/A#/G# candidates rise; D# major/sus4 score highest in this broad window. | Could be a chromatic/pitched effect, passing region, true harmony or analysis error. A B-natural-minor-only reconstruction might omit a recognizable color here. |
| 12.00–14.00 s | Strong B/F# candidates; E sus2 and B sus4 share the same pitch set and are tied by design. | Verify which root is actually present and whether this is a return of the opening figure. |
| 14.00–30.00 s | Busier visual onset region; chord candidates vary through Bm/G/D/suspended E-A colors; D# appears around 22–24 and 28–30 s. 2-second RMS is about −20.5 to −20.2 dBFS. | Recover the full bass/hook repetition and the location of chromatic replies. Do not substitute a stock four-chord loop and call it the song. |
| 30.00–40.00 s | B/E/F# family continues with G around 38–40; conspicuous changes in harmonic density near the late region. | Author a repeatable late phrase after inspecting the exact note cadence. |
| 40.00–42.68 s | E/B-related upper candidates. A 20 ms RMS scan finds a low-energy tail at 42.42–42.68 s (<0.02 RMS, ~−34 dBFS). | The supplied clip is not automatically a seamless loop despite its tag. Compose/verify a cadence and pickup explicitly. |

The strongest foreground-note **pitch-class** sketch to audition is an opening F# / B anchor, then F#–A–D, with E/G/B replies. The supplied measurements do not establish a monophonic melody from that sketch. Exact note choices, rests and octaves are still the important remaining composition work.

Mono decoded peak is 0.5851135 (about −4.66 dBFS), RMS 0.0913079 (about −20.79 dBFS). These are analysis-WAV values, not source stereo peaks, LUFS, in-engine headroom or a target mix setting.

## Native score design

Use one authored event score and one AudioContext beat clock across title, corporation hub, exploration and combat. Event times are quarter-note beats, durations are beats, pitch is MIDI, and velocity/voice/articulation are explicit. Title, hub and area variants share a manually checked motif and harmonic phrase map. Render with oscillator/noise synthesis; the MP3 remains an offline composition reference.

Proposed data fields are shown in `score-plan.json`. Its motif and chord examples are **new audition drafts**, not transcribed source notes. Do not promote them as a completed recreation. Replace the examples with listening-checked score events before marking the title song matched.

Suggested voices:

- **Chip lead:** bounded periodic wave or square oscillator shaped by a gentle low-pass; 3–6 ms attack, authored 60–220 ms gate/release, no global random notes. Use pulse-width/color variants and occasional authored octave responses. Confirm the source's actual lead prominence and octave first.
- **Clockwork arpeggio:** triangle/sine blend or a softer pulse, sixteenth/eighth-note authored patterns, low velocity, short gate. A 2–3 ms note attack and limited polyphony prevent a harsh continuous block of sound.
- **Bubble voice:** sine/triangle with a short two-point exponential pitch glide and round low-pass envelope. Author each glide/destination rather than accidentally quantize every effect as a chord tone. The metadata suggests this role; it has not been heard/accepted here.
- **Bass:** triangle plus quiet sine/sub fundamental, short attacks and musically timed releases. B2 / F#3 are initial tuning references if B minor is accepted; actual source bass pitches need listening.
- **Glitch/percussion:** deterministic generated noise buffer, high-/band-pass filters, authored mini-bursts, quiet high ticks and a sine pitch-drop kick. These buffers are generated by the engine, not decoded recordings. Avoid copying the source's MP3 waveform into generated buffers.
- **Ambient pad:** quiet sine/triangle chord tones, 150–300 ms attack, 300–600 ms release, no sharp PWM. Let sparse motif fragments carry identity in hubs/exploration.
- **Combat string:** the existing shaped palm-muted voice can add a layer at 166 BPM; it must follow the checked score harmony. Keep its authored syncopation distinct from the lead so it does not obscure the recognizable hook.

A provisional B-minor chord audition set is Bm `[47,54,59,62]`, G `[43,50,55,59]`, Em `[40,47,52,55]`, A `[45,52,57,61]`, with F# major `[42,49,54,58]` reserved for a confirmed dominant/chromatic turn. This is a usable **composition vocabulary**, not a recovered progression. The measured D#/G#/A# region may require different passing material after listening.

## Shared clock and transitions

1. On the launch gesture, unlock the existing AudioContext and start the authored title score. No fetch, MP3 decode or independent title playback source is required. Route title through the same music/master controls, compressor and pause lifecycle as world music.
2. On Continue, move to the selected corporation hub. Keep `beat`, chord phase and voices already in release. Lower the title lead/percussion immediately over ~150–300 ms and bring in the hub's common B/F# pad over ~0.8–1.4 s. Change full chord voicings at the next beat/bar boundary. Begin a two-bar tempo descent to 120 BPM; do not restart the music clock at zero.
3. Use 120 BPM for exploration. A sparse melody fragment, arpeggio answer and shared bass/chord cell preserve song identity. Area/corporation variants alter orchestration, register, velocity and authored replies rather than unrelated random note material.
4. When the authoritative hostile/chase state triggers combat, queue a one-bar tempo ramp 120→166 BPM on the next quarter-note. Fade percussion/strings in over one bar. Keep beat phase continuous and avoid resetting the harmony when tempo changes. Boss adds voicing/rhythm intensity through the same clock.
5. When chase/combat ends, hold the release intent briefly (for example two beats after authoritative threat goes quiet) to avoid constant mode flapping. Fade combat voices over ~1–2 bars and use a two-bar ramp 166→120 BPM. If threat returns, retarget from the clock's current tempo/beat instead of restarting a ramp or cancelling audible notes abruptly.

Timing implementation: integrate tempo in beat space. For a ramp from `B0` to `B1` over `N` quarter beats, let `B(b) = B0 + (B1−B0)*(b−b0)/N`. Scheduling a beat interval uses `dt = 60/k * ln(B(b1)/B(b0))` for nonzero slope `k`, or `60*(b1−b0)/B0` when constant. Split intervals at ramp endpoints. This makes a “one-bar ramp” last exactly four beats while notes stay on the shared beat grid. Do not merely replace `60/112/4` with an abrupt different interval.

One-bar 120→166 ramp is about 1.693 seconds; the reverse two-bar ramp is about 3.386 seconds. Those are authored transition durations, not source measurements. Smooth the musical density and instrument gains in addition to tempo.

Harmonic transitions need a boundary policy: carry the current shared chord root/third/fifth through the fade, admit newly voiced full chords at the next bar, and initially sustain the common B/F# or single root if a variant uses a chromatic phrase. A title ending at arbitrary time must not stack a new hub chord against an incompatible old release. Keep at most two theme arrangements scheduled during a fade and respect the existing music voice limit.

## Existing engine constraints and integration consequences

- `apps/game/src/title-theme.ts` currently implements a nullable URL fetch/decode/loop source. This is the runtime-recording path the user's request supersedes. An authored title score should use note/voice data instead.
- `apps/game/src/audio.ts` currently schedules 16 steps per bar at a fixed 112 BPM, using eight A-minor chord bars. `scoreStep` returns during title, while title audio bypasses the dry music bus and uses its own gain into master. The new title score needs one coherent lane/clock, not a second scheduler.
- `apps/game/src/audio-state.ts` already receives authoritative combat, area and corporation state; `cleanState` suppresses combat in haven/site. Distance or visual effects should not create chase music. Use the actual available threat contract when defining the ramp trigger.
- If the accepted song center is B minor, retune the existing A-minor harmony and engine A–E bed (currently MIDI 45/52) coherently, or explicitly transpose the accepted source motif into the existing key. Pitch-preserving recreation argues for B minor. A drone left in A–E can clash with a B-minor title/hub score.
- Existing `scripts/qa/audio-review` can render actual OfflineAudioContext output and export float WAVs with peak/RMS/nonfinite checks. Its current source preparation knows `title-theme.ts`, `audio.ts` and `audio-state.ts`; any new score module must be added to that preparation and served module allowlist by root.
- Native authoring must remain renderer-independent and use bounded voices. The game cannot depend on the studio service or this work directory.

## Completion gates still open

1. **Composition/listening:** directly audition the supplied source and native draft side by side; identify the principal hook, exact octave, rhythm/rests, bass roots, chromatic region and phrase cadence. Mark whether the recognizable song is recreated. The present automatic candidates are insufficient for that claim.
2. **Offline native audio:** render title, all three hub arrivals, 120 BPM exploration, chase ramp to 166, boss and return through the actual production AudioEngine with source/module hashes. Export actual WAVs and inspect peaks, nonfinite samples, clipping, voice count and note/tempo timestamps. No surrogate Python synth output should stand in for this engine check.
3. **Transition continuity:** exercise Continue at multiple positions in the title phrase, rapid chase/release/re-chase, mute/unmute, pause/resume, return to title and death. Inspect the real scheduled beat/tempo/harmony and listen for clicks, phase resets, stacked chords and stuck voices.
4. **Physical listening:** listen on the actual output device at representative settings with game effects; confirm balance, fatigue, recognizable motif and smooth tempo transitions. Record real evidence, leaving this gate open until completed.

Current acceptance state: **reference analysis complete; source transcription, native implementation, perceived match, production verification and physical listening remain unfinished.**
