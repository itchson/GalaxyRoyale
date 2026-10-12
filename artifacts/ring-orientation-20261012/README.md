# Seeded ringed-planet orientation

The scoped runtime change rotates the entire cached back-ring/body/front-ring group by one bounded seed angle. Materials, cells, parallax, culling, budgets and RNG stay unchanged. Root's integrated source303659 passed839 native tests with11 explicit exporters ignored, strict native/WASM Clippy and the normal dual build.

Root inspected actual1280x720 desktop frames at verified1.6 world zoom. A/B have controlled before/after WebGL2 captures; after A/B also have actual WebGPU frames. C has a valid after frame only: before-c-webgl2.jpg did not establish target presence/zoom and is retained as a failed comparison. before-a-default-webgl2.jpg is an uncontrolled initial tall-window/low-zoom view. Do not use either as an equivalent baseline.

Art inspected actual both-backend stills: group occlusion and intended orientations are readable. The captures differ in time, minor camera drift and nearby transient objects; they do not claim exact pixel/color parity or new mapped-surface acceptance. after-build.json identifies compiled303659, not later material shader source. Actual STATS text is preserved beside the frames. Future albedo diversity and physical-device motion remain separate gates.
