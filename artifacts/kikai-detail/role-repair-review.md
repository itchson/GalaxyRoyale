# Corporate color-role repair: independent source and visual review

Scoped acceptance of Art repair `1b5e4087451789c3cc0c22fc04d6d119f5427869` / final atlas SHA256 `31fa6a1b04d83f3d5fad377562e73c6ad35530b028626a1136c5b56afd9ea661`. No blocking defect was found in the inspected repair. This addendum does not rewrite the earlier high-detail art review.

The original role-generation rule treated any red-dominant occupied cell above row 85 as horn crystal. Actual Root WebGPU captures showed a blue forehead band for TALO RENKO/HUGO and a green forehead patch for DORO HUGO/PAN. Those captures were inspected as existing images; this reviewer did not operate their browser. Pins:

- `artifacts/kikai-detail/talo-webgpu-1280.jpg`: `5eba50ac7bcad16a5a76a07ec33f4b27d39e353188f99afff54ef8c5eec8e0ee`.
- `artifacts/kikai-detail/doro-webgpu-1280.jpg`: `e8fce7d46bd8697831aa88b68566a3c3c332e0ba4078485d968cc0c20aa1d426`.

The repair limits each crystal to an authored spatial region and an 8-neighbor component rooted in an actual saturated crystal cell. It excludes known warm facial skin, eyebrows and neck trim. Independent Git-blob comparison proves every palette and sprite cell array is unchanged, along with all three hair masks. Horn masks shrink DORA 1,139→942, DORO 1,219→864, TALO 1,567→1,009; signal masks 740→737, 622→621, 664→664. Submitted geometry, authored chest planes, facial poses and full-portrait cell costs therefore remain unchanged.

Six inspected skin probes are no longer in hair/horn masks; six positive crystal probes remain in horn masks. The actual native test adds an independent semantic oracle: source forehead RGB retained in all 27 extracted profiles, real crystal colors retained/recolored in 54 reads. Its original 18/27 failure and final success are preserved, so the test does not merely endorse the corrected mask.

The corrected software contact sheet `assets/ui/kikai/detail/corporate-role-review.png` (SHA256 `feb39aa56110bb1d48fc798207fabdc55224e6a92707b3a4107223b2f878f7a6`) was visually inspected. The failed DORO HUGO/PAN and TALO RENKO/HUGO combinations now show warm source-color forehead beneath the fringe while crystal facets retain their secondary colors. No obvious face-overlay displacement is visible in that sheet. DORO/TALO planar torso silhouettes remain as previously reviewed; this is a scoped visual judgment, not a pixel test purporting to prove anatomy.

Art's reported 13 Python checks are peer evidence, not independently rerun here. Own execution: two actual native extraction tests and strict native game_web Clippy. Exact sources/logs/comparison are pinned in `evidence.json`.

Pending: Root's rebuilt GPU/GL screenshots in these exact combinations; compact-portrait/finite physical-sampling review; full-mask visual inspection beyond these probes; cold cache timing/memory; human visual and physical-device acceptance. The 60,000 UI cap, atomic portrait admission and exclusive-menu fix are unchanged. No cap increase or role-generation rewrite was introduced by this reviewer.
