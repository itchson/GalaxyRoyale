# Neutral sports UI chrome refinement

Source freeze `6153bd211491775e9a34a23185bd501a9f3d6b41`, parent `ceb784a953a028e505b0c8f28fcb5c28f124adce`, branch `codex/sports-ui-skin-refinement`. Only `crates/game_web/src/sports_ui.rs` and `sports_ui_tests.rs` changed. Tracked tree clean. No renderer/shader, shell, HTML/CSS, title, MEISHI, portrait, icon atlas, gameplay, storage, co-op, shared manifest, Studio, server or browser edits.

## Concrete cause and implemented change

I viewed Root's saved `artifacts/space-sports-style-20261012/settings-webgpu-after.jpg` and the actual `assets/ui/title-mark/source-cells.png`. The Settings image has a green-looking full-panel backing. Sampled persisted JPEG interior pixels were approximately RGB(0,20,19). This is not a new browser capture by me.

Source explains the condition: `PixelUiBridge` marks the root active when it contains the focused control; the prior chrome changed the full panel fill to RGB(12,18,24). The physical UI attachment is linear `Rgba8Unorm`, with `pow(source_sRGB,2.2)` before storage. Independent eight-bit rounding maps that very dark color to (0,1,1), producing the cyan/green bias on presentation. No extra panel override or world-light response was necessary to explain it. This is source/math correspondence, not an instrumented GPU diagnosis.

Panel backing is now opaque equal-channel graphite RGB(18,18,18), independent of inherited focus, selection or corporation. Disabled neutral shading also uses equal channels. Real controls retain their independent hover/selected/category accents; corporation semantics and measured label/hit rectangles are untouched. Neutral alloy/graphite shades have less blue wash.

An original cached 48×12 competition shoulder adds sloped platinum facets, a recessed enamel channel and a small gold award locator. It fits continuously into the panel's existing top gutter; no minimum cell or LOD reduction. The top metal shoulder is asymmetrically segmented, with a short enamel inset instead of an oversized default accent rail. Other panel art stays quiet around the text. No repeated logo, added animation clock, new atlas or dependency.

## Actual checks and costs

- `cargo test --locked -p game_web sports_ui:: -- --nocapture`: **10 passed**, including three new tests. It checks authored landmarks, inherited panel focus/corp invariance, an independent old/new eight-bit quantization model, tiny fractional source preservation, reflected/sheared partial clipping, atomic both-quota failure, neutral metal/state accents and the joined real UiScene fixture.
- `cargo clippy --locked -p game_web --all-targets -- -D warnings`: exit 0.
- Existing native exporter test with an explicitly scoped `GALAXY_SPORTS_UI_REVIEW_OUTPUT` wrote `packets/menu.json`; **one passed**. Environment override removed afterward.
- Actual retained packet: **47,749 cells / 266 transforms**, all complete components equal their isolated extraction. Both 60,000 submitted-cell and 120,000 work caps remain unchanged; 128-transform batching is respected.
- Panel: **1,557 cells**, +389 from three extra rail surfaces and 386 authored shoulder cells. Button: **348**, unchanged. Full tiny/ordinary chrome counts remain equal; hidden alpha-zero cells still consume both quotas.
- `git diff --check` passed. Rustfmt touched only the two owned files.

`render.py` reused the prior explicit native packet software renderer and saved `native-software-menu.png`: four-times supersampled polygons, BOX reduction, flat unlit source colors. I inspected that image: neutral backing, quiet perimeter, legible untouched text, small header motif and dominant portrait. It is **native extraction plus software approximation**, not actual DOM/GPU sampling or a claim of user approval/full sports-reference parity. In particular it does not model the physical linear attachment's quantization.

Root owns combined checks/build and actual WebGPU/WebGL2 Settings, RAICHO and compact scrolling/Close acceptance. Physical devices, broad visual consistency across every asset and subjective legibility remain separate gates. No full game build or browser/server was run here.
