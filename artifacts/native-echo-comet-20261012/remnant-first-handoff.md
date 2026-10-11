# Open supernova ejecta — native art handoff

Frozen source 937c161573b3ed4c88bcc38cf699e9bd24c7043c, parent
c87e482d32e8742294edd97c1fbb2a8a0a803e1a, branch codex/open-remnant-art.
Only crates/game_core/src/cosmic_background.rs changed: cached shell sampler,
one morphology regression and one explicitly ignored native proof writer.
No shader, renderer ABI, gameplay, UI, other background class, density/cap,
shell/source palette or shipping asset was changed.

Root's actual current WebGL2 capture remnant-webgl2-before.jpg showed a thin
reddish almost closed perimeter behind the craft. Actual unobscured before
WebGPU/WebGL2 frames are retained separately. This was a reproduced display
defect, not an inference that real remnants can never have shells. NASA's
[Cassiopeia A NIRCam description](https://science.nasa.gov/asset/webb/cassiopeia-a-nircam-image/)
was actually read: its shell contains fine filaments and knots with material
within and outside it. That is morphology reference for original stylized
art, not a scientific simulation, copied photograph or real light source.

The cache now samples unequal open fragments, narrower spurs and sparse
interior knots from authored variable-width paths and correlated shading.
It keeps the original eight shell colors, six core colors, core sampler,
128/64 shell and 24/12 core grids, half/unit source scale, existing low-alpha
material, group motion/pulse and two transforms. No repeated closed contour
is drawn. The angular gap already tested by the engine stays empty, with two
additional broad routes to sky. Source outer bounds are deliberately unequal
and slightly broader than the old radial edge, within the unchanged source
canvas and conservative180 culling bound; occupied silhouettes are not equal.

Actual total costs: before857/210, after873/208 full/reduced. The unchanged
core costs76/16; shell781/194 becomes797/192. Submitted alpha-zero cells remain
charged. Whole-object admission and existing2,200/1,100 motif ceilings,
custom8,000/4,000 budget/star reserve and regional gas limits are unchanged.
No per-frame heap allocation was added: this sampler runs at cache creation.
Cold LazyLock initialization and FrameBuilder growth are not allocation-free
or newly measured performance claims.

## Actual checks

- The new morphology regression first failed on unmodified c87 production
  sampler, with18/781 interior cells and one broad exit. Baseline native proof
  used that sampler plus only the added assertion/writer instrumentation.
- Initial candidate had interior structure but only two broad openings and
  failed the same assertion. It was refined; neither threshold was relaxed.
- Final actual shell has212/797 and49/192 interior cells, three broad exits in
  each density. The test measures emitted cache cells, not path constants or
  an exact output count. The two constraints address the observed perimeter.
- cargo test -p game_core --locked cosmic_background -- --nocapture:
  **29 passed, zero failed, four explicit proof writers ignored**. Existing
  checks cover source palettes/extents, relative affine/fractional cells,
  far-camera corners, single parallax, exact Reduced Motion, small-step
  animation, whole-quota rejection and128-transform batch boundaries.
- Baseline and final explicit export_actual_seeded_remnant_packets invocations
  each actually passed once, writing four packets each with three zoom views.
- Strict cargo clippy -p game_core --all-targets --locked -- -D warnings,
  cargo fmt -p game_core --check and git diff --check passed.

No full workspace/normal WASM build, browser execution, server, synthetic GPU
screenshot or phone/soak claim was made by this specialist. Root owns rebuilt
WebGPU/WebGL2 visual acceptance. Engine independent review remains attributed
separately; its actual exchange was source/packet inspection, not a meeting.

## Native/software evidence and reproduction

before/ and after/ contain actual native FrameBuilder/CameraView JSON packets,
including local cells, both affine bases, RGBA, materials and corners.
Eight packets /24 views /51,552 transformed corners are retained. The source
seed/grid/hash/origin/camera and transforms are unchanged; core tails agree
exactly in every before/after view. render_review.py produced the actually
inspected native-software-contact.png and software/*.png with fourfold offline
flat-alpha sampling over near-black. This is not WGSL, GPU lighting/compositing
or exact screenshot parity. Pose times0/18 differ from Root's browser capture;
Root's before screenshots prove form, not a time-matched moving comparison.

PROFILE must actually report seed871238449. Motif grid[-3,1] has hash1888154543;
camera=origin/.08 gives[-35332.82,22642.145]. Centered QA command:
WARP -35332.8203 22642.1445. Prefer unobscured comparison at zoom1.6:
WARP -34932.8203 22642.1445; the unchanged +400X camera offset places the motif
32 world units left of the craft. QA WARP suppresses progression rewards and
abandons carried material; this is not naturally earned/travel acceptance.

Explicit native writer requires an owned absolute REMNANT_PROOF_DIR, then
cargo test -p game_core --locked export_actual_seeded_remnant_packets --
--ignored --nocapture. Ordinary focused tests do not write proof outputs.
manifest.json/verify.py pin source, retained raw evidence and current baseline
browser image identities; reproduction never overwrites those inputs.
