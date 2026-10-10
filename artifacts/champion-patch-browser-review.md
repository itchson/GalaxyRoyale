# Champion patch browser composition addendum

Independent inspection of root-supplied foreground captures and actual DOM records, recorded 2026-10-10. No blocking patch composition defect was observed at these captured extents. Earlier native baseline, candidate, and production reviews remain immutable and separate.

## Observed scope

Root captured the current normal WebGPU build at 1280×720, a requested 853×480 viewport whose record reports 854×480, and 640×360. Root also supplied an actual WebGL2 victory capture and 32-logical ship measurements. I opened the six listed JPEGs with `view_image` at original detail and read their paired metadata. I did not operate the browser or obtain a GPU readback.

The mixed ownership state comes from an explicit strictly valid supplied QA save fixture (`work/scar-browser-fixtures.mjs`) based on a previously exported profile. The alternating champion patches and all-eight-plus-league-win victory state were supplied for this composition check; they do not demonstrate natural earning, tournament completion, or save progression. Root reports Reduced Motion checked for these captures. Static images do not prove animation timing.

| Supplied evidence | Actual observation |
|---|---|
| `champion-emblems-webgpu.jpg` / `-dom.json` | 1280×720, DPR 1.25, CSS UI scale 2.4, primary canvas backing 1600×900. The visible Lyra needle and Korr shield retain their distinct occupied shapes beside readable card headings. The eight measured icon boxes are approximately 24.000000954 logical units on both axes. |
| `champion-emblems-webgpu-scrolled.jpg` / `-scroll-dom.json` | Actual scrollTop 324. The top clip crops Nightrail and Prism; the bottom clip crops Windrunner and Ironwake. Their remaining cells stay in the scrolling content area and do not cover the fixed tabs/footer. |
| `champion-emblems-fractional-webgpu.jpg` / `-dom.json` | Recorded 854×480, DPR about 1.000000015, UI scale 1; main canvas CSS width about 853.600037 and backing width 854. This first image retains **bottom scrollTop 389.600006**, so the champion icons are outside the clip and absent. This absence is consistent with its metadata and is not a failed icon render. |
| `champion-emblems-fractional-webgpu-scrolled.jpg` / `-scrolled-dom.json` | Despite the filename, this record has **scrollTop 0**. The image visibly shows intact Lyra/Korr patches at measured 24×24 logical boxes, with a two-column roster. |
| `champion-emblems-narrow-webgpu.jpg` / `-dom.json` | 640×360, scale 1, one-column roster. The visible Lyra icon is intact and separated from its text and footer. The measured clip is approximately y=135.800003..237.200005 logical units; the next card starts below it. |
| `champion-ships-webgpu-dom.json` | Ten ship-selection boxes measure approximately 32.000001272 logical units at scale 2.4. This is geometry/state evidence only; no paired ship image was supplied for this addendum. |
| `champion-royale-webgl2.jpg` / `-dom.json` | Actual scrollTop 1008. The ninth gold crown/star/laurel emblem is visibly intact above GALAXY ROYALE. Its value is 8 and authoritative active flag is true, measured approximately 24.000000954 logical units at y=139.791667, safely inside the content clip. |

The earlier `champion-royale-webgpu.jpg` is not used as visible-ninth-emblem proof: root reports it was overscrolled into the standings. Likewise, DOM presence of all eight champion elements does not imply all eight were visibly rendered in one capture. This set visibly covers six base emblems across the WebGPU scroll views and the ninth victory emblem in WebGL2; it does not establish visible browser presentation of Vexa/Atlas in these specific images.

## Measured fit and crop checks

Command executed:

```powershell
python work/champion-patch-engine-review/browser-dom-analysis.py
```

Seven supplied records contain **59 actual icon boxes**. Every measured 24/32 extent is inside the production bridge's narrow `1/64` logical tolerance. The maximum measured discrepancy is about **0.000001272 logical unit**. The earlier exact production-expression proof established that this normalization preserves the measured center and does not round unrelated font/text rectangles. These records provide actual browser measurements at the observed scale; the earlier 108 numeric fixtures remain separate evidence.

The 1280 partial-scroll clip is `[28.000002,115.395838,477.333323,114.770838]` in full logical-screen coordinates. After preserving measured centers and normalizing extents, the authoritative atlas predicts center-visible occupied-cell counts for selections 0..7 of **[0,0,219,202,177,248,0,0]**. The screenshot agrees on which shapes are absent, top-cropped, or bottom-cropped. These counts are a metadata/atlas prediction, not a count measured from compressed browser pixels. JPEG compression and fractional screen scaling do not support exact pixel-color or center-coordinate claims.

No opaque patch visually escapes into the fixed menu controls in the inspected partial-scroll images. This observation does not strengthen center clipping into a GPU scissor: parts of a cell quad may cross a fractional edge by design. The measured active flags alternate true/false in the mixed fixture and agree with the source's dataset-only branch; the static captures do not independently test hovering/focusing an unearned patch or demonstrate its color sweep over time.

## Remaining acceptance boundaries

These captures establish representative normal browser icon presence, measured fit, partial-scroll containment, and visible ninth selection in the observed WebGL2 scene. They do not prove animation timing/human preference, all device/DPR combinations, full natural campaign progression, complete ship-gallery presentation, phone performance, or published build acceptance. No production, candidate, API, or frozen prior evidence was edited.
