# Open remnant gas-shoulder amendment

Frozen source: `8e6fabdfea0aca352e60e74970da30f15af7ae9a`, parent `937c161573b3ed4c88bcc38cf699e9bd24c7043c`. Only `crates/game_core/src/cosmic_background.rs` changed. Tracked checkout is clean. The first candidate's report and packets remain historical and unmodified.

## Actual finding and bounded change

Root's persisted b98 WebGPU `artifacts/native-echo-comet-20261012/remnant-offset-webgpu-final.jpg` shows isolated thin brown contours with mostly empty space around them. I inspected that file. The first candidate removed the ring but had not met visual acceptance.

The existing cached shell sampler now adds five unequal filled density pockets around selected fragments, with correlated coarse/fine texture and faint alpha bands. Bright cells occur in limited knots; long contours are quieter. These are original procedural native pixel cells using the existing palette alpha gradients, not radial soft-cell material, new lighting, imported imagery or a physical gas simulation. The palette, core sampler, source dimensions, two parts, motion, seed selection, parallax, quality identity and render path are unchanged. No other background or gameplay source changed.

The source direction retains dark routes between unequal ejecta rather than restoring an outlined shell. NASA's [Cassiopeia A NIRCam description](https://science.nasa.gov/asset/webb/cassiopeia-a-nircam-image/) was read during the first pass; its filament/clump/wispy-outskirts description grounds the art direction without implying that all real remnants lack shells.

## Actual checks

| Check | Actual result |
| --- | --- |
| New broad faint-density native regression against 937c sampler | Failed: zero broad faint centers; 521/797 shell cells were bright |
| `cargo test -p game_core --locked cosmic_background -- --nocapture` | 30 passed, 4 explicit exporters ignored |
| Explicit existing native packet writer | 1 passed; four actual FrameBuilder/CameraView packets |
| `cargo clippy -p game_core --all-targets --locked -- -D warnings` | Passed |
| `cargo fmt --all -- --check`, `git diff --check` | Passed |
| `python work/open-remnant-gas-amendment/verify.py --draft` | Passed: 17,064 cell records / 68,256 finite corners |

The new regression checks actual occupied low-alpha neighborhoods at least two world units across in both source grids, faint-cell prevalence and sparse brighter cells. It is not an exact output-count assertion. The retained structural regression still requires interior ejecta and at least three broad sky exits. Existing focused tests cover whole-group admission, source scaling, reflected/sheared transforms, far coordinates, quality identity, small-step motion, Reduced Motion, gameplay independence and existing custom budgets.

| Source measurement | Full | Reduced |
| --- | ---: | ---: |
| Complete remnant / unchanged cap | 1,416 / 2,200 | 347 / 1,100 |
| Shell occupied cells | 1,340 | 331 |
| Unchanged core cells | 76 | 16 |
| Broad faint neighborhood centers | 145 | 38 |
| Source alpha <=23 cells | 1,085 (81%) | 263 (79%) |
| Source alpha >=31 cells | 190 (14%) | 52 (16%) |
| Broad angular sky exits | 3 | 3 |
| Transforms | 2 | 2 |

Custom 8,000/4,000 and regional 6,000/3,000 limits were not changed. No new per-frame allocation or new material/shader path was added; the pocket evaluation runs while building the existing cached bodies.

## Evidence and source correspondence

`before/` contains the four actual 937c native packets reused byte-for-byte from the immutable first candidate's `after/`. They were not newly exported for this amendment. `after/` is the new native export at this source freeze. Eight A/B packets contain 24 zoom views (0.6/1.0/1.6, times 0/18, both densities). The verifier confirms exact core-tail colors, transforms, local cells and corners; unchanged seed/origin/camera/time/material metadata; all finite corners; identical source contents at each zoom; and unchanged caps. Runtime production source outside `remnant_filament` is byte-equivalent after line-ending normalization.

`native-software-contact.png` and `software/after-lowfalse-time18-z1.6.png` were actually viewed. `render_review.py` uses actual native corner/color packets with a 4x flat-alpha Pillow/BOX approximation on near-black. It does not run WGSL, HDR/postprocessing or either GPU backend. In that software view the long contours are subdued and wider irregular faint pockets are visible, with open dark gaps. The result is intentionally very faint; actual GPU visibility and natural depth remain pending.

Retained logs: `baseline-fail.log`, `focused-initial.log`, `focused-final.log`, `native-export.log`, `clippy.log`. `manifest.json` pins this source, the packets, software images, scripts, report and Root's actual rejected b98 frame.

## Root reproduction and remaining gate

Actual native seed `871238449`, motif grid `[-3,1]`, hash `1888154543`. Center WARP `-35332.8203 22642.1445`; unobscured offset WARP `-34932.8203 22642.1445`, zoom `1.6`, same profile seed. These coordinates preserve the prior comparison. Root must rebuild normal artifacts before observing this source and retain actual WebGPU/WebGL2 comparisons. Browser pose times need not be equal, so those are form/visibility comparisons rather than time-matched motion proof.

No browser, listener, shared build, human approval, physical-device, performance or soak acceptance is claimed by this amendment. Root's new GPU comparison is required before visual acceptance.
