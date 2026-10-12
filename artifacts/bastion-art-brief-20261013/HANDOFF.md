# Bastion source-art brief

Baseline is clean `4bbcd7b98b5c242e1c6f3c614bae4bf50f6949c9`. Only this work directory was written. [Actual native/software contact](bastion-dart-native-contact.png) compares Bastion's existing 40-grid and ordinary 0.7-scale 28-grid with the refined Dart source-art benchmark. Author inspected it and the existing packed PAN/RAICHO benchmark contact. These are flat software views, not moving Game/GPU/human acceptance. Native groups have identity orientation; the gameplay row uses the actual ordinary actor cell scale at nominal zoom1, not a captured camera or live actor.

Bastion's broad silhouette, warm graded orange plates and central core already read. The concrete consistency gap is its lower pod rims merging into purple recesses at28px, with soft shading obscuring the smaller mechanical separation. The approved Dart/PAN/RAICHO discipline organizes compact plate bevels, darker recesses and sparse alloy/mechanical accents. This observation does not justify removing gradients or copying the benchmarks' larger grids.

Root inspected the actual contact and approved this isolated candidate brief: retain warm graded plates/core and silhouette; prioritize compact alloy rims around the paired lower pods plus sparse recess/vent clusters. Avoid broad gray strips, flat recolor, new pulsing sites or a brighter full perimeter. A future source-art candidate must be viewed before integration; this approval is the bounded brief only.

Required invariants: exact40×40 grid,938 occupied-cell mask,64-entry palette, all83 original index4 sites and the empty original index6/33 site sets. Preserve every other atlas byte, centered pivot, actor scale, animation, collision/body budgets and all gameplay source. Do not recategorize the fixed amber signal cells as a dark canopy. Work within non-signal cells using short2–3-cell bevel/rim/shadow clusters and existing palette roles; retain the current orange plate gradient and avoid uniform bands.

Actual commands:

- Existing asset-pack provided-palette indexed import produced `pack.json` from the exact exported `source.png`/`manifest.json`; source sprite and palette equal the current atlas.
- `cargo run --offline --locked --manifest-path scripts/asset-native-preview/Cargo.toml --target-dir work/bastion-art-quality-review/native-target -- work/bastion-art-quality-review/pack.json bastion-review work/bastion-art-quality-review/native.json`: exit0; two actual native PixelBody/FrameBuilder frames,938 cells/one transform each.
- `python scripts/asset-audit.py --pack work/bastion-art-quality-review/pack.json --extraction work/bastion-art-quality-review/native.json --output work/bastion-art-quality-review/native-audit.json`: exit0; exact cell colors/order, native grid zero corner error, 0.7-scale maximum corner/bound error5.73e-7 within f32 tolerance.
- `python work/bastion-art-quality-review/render_contact.py`: exit0; actual exported hard quads rasterized by pixel-centre/half-open bounds without smoothing, lighting or glow.
- Root's actual `scripts/qa/freeze-native-proof.mjs` copied the scoped extractor and linked engine library into immutable hash-named `proof-binaries/`. Compiler inputs, flags, versions, depfiles and command logs remain separate; the copy utility does not infer provenance.

Studio task `task-864e2556` has actual running observation `run-68877cfd` at revision1095. The actual Root brief review and completed inspection observation are recorded separately through the API; task status/criteria are not self-approved. Production edits, normal package build, listeners, browser, GPU/backend presentation and human moving-play acceptance were not performed. The next candidate will have a separate branch/work/target scope; frozen Dart/nebula artifacts are not reused.
