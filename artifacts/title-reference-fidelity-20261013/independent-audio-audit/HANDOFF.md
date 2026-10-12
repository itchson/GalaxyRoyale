# Frozen engine audio source/timing review

Author source has not been edited, and no tracked runtime source changed. Own branch remains `codex/eight-pilot-streaming-review` at `4ab10c0a7018501ebfaa7dd978101ba0d979f6fb`; these audio review files are work-only. No commit, build, listener, upload, browser or audition was performed for this assignment.

`REVIEW.md` gives the source/API/input audit, generated timing result, additive coefficient mapping and minimal continuous-event scheduling requirements. `ADDENDUM-FULL443.md` pins the later20-file freeze, actual18 coefficient checks, low-event coverage, descriptive spectral checks and local-phase/contour-knot cautions. `calibration/HANDOFF.md` is the bounded actual inference record; its raw model arrays and original generated gates remain available.

Concrete integration blockers are blind raw-contour tuning (known steady tones falsely get median+33.333 cents) and attempting fractional events through the current integer equality score API. The source-derived fixed-pitch controls themselves have a valid periodic synthesis path. They still need recovered/cleaned notes, percussion/voice labels, loop/cadence and browser/ear acceptance; neither fitted coefficients nor RMS/PSD comparison proves resemblance.

Actual independent executions:

- `audit-hook8.py`: frozen40-file verification, exact9 coefficient checks, reference-prefix distinction, finite output/levels; exit0.
- `calibration/run-calibration.mjs`: one43-second generated fixture, pinned official TFJS CPU API/model,14 exact pitch matches in both decoders; exit0. No real-song input.
- `calibration/verify-calibration.py`:13 provenance pins, generated PCM/ground truth and3698-row raw shapes, actual note-clock errors and false bends; exit0, no inference rerun.
- `audit-full443.py`:75 author frozen pins,443/23low coverage,18 exact coefficients, two physical outputs/prefixes, actual Welch-share recomputation and transient-data bounds; exit0, no synthesis rerun.
- `audit-contour-clock.py`:443 timed events' unique converter inversion/durations,45 boundary crossings, up to10.0613ms uniform-knot error and10.4025us rounded phase-origin offset; exit0, no decode/model rerun.

`MANIFEST.json` lists all own raw evidence bytes and the separate external source/input pins. `verify-manifest.py` checks them plus every declared frozen author file. Canonical Git blobs and post-execution installed-dependency observations are not silently substituted for raw file pins. The original Python import failures are retained externally and pinned, with no successful Python-model claim. No production action follows automatically from this source review.
