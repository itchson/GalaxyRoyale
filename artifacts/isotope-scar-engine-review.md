# Isotope Scar engine integration review

Native engine handoff accepted for a future integration pass. No source defect was found in the frozen work-only extractor. Production, the art candidate, shared manifests and deployed evidence were not modified. Browser rendering, natural seeded visibility, human art acceptance, frame timing and physical phones remain Pending.

## Pinned source and actual execution

The candidate manifest is `work/radiation-background-art/frozen-hashes.json`, SHA-256 `ffe62aa60b1b7b698dd73b117f7ecec127ebdf55a21f984275dac901b0da6a34`. Its extractor is `scar.rs`, SHA-256 `2ddba8e85b64b7aee8194482796952356d320e8bdc915871b218e4ab89965f22`; its atlas is `5c3d293f47d62d4639207aea9d71c577f6c913cdeb5f9ecad76117febfec73ce`. The manifest's complete source/evidence/dependency/context identities were checked before compilation and after execution. `candidate-fingerprints.json` records them alongside this review's executable probe inputs.

The existing region baseline is SHA-256 `6d76c0f42e49c1a1adfd55d144e27536829c83e08976a80d0b0517ed8c977c1f`. Baseline probes compile an exact owned copy with a test module; candidate probes link the actual frozen `scar.rs` directly. Both use existing compiled `engine_core`, serde and serde_json libraries. No duplicated renderer, new dependency or manifest was introduced.

Commands, run from the repository root:

```powershell
./work/radiation-background-engine-review/build-baseline.ps1
./work/radiation-background-engine-review/baseline-probes.exe independent_background_review --nocapture
./work/radiation-background-engine-review/build-candidate.ps1
./work/radiation-background-engine-review/candidate-probes.exe --nocapture
python work/radiation-background-engine-review/prepare-candidate.py
```

Observed results: **2/2 focused baseline tests** and **9/9 candidate-linked tests** passed. The latter comprises three independent packet/projection tests and the author's six tests. The candidate-linked probe/test compile also passed strict `clippy-driver -D warnings`. Logs are `baseline-proof.log`, `candidate-build.log` and `candidate-proof.log`; their hashes, source and executable hashes are retained in `review-evidence.json`.

## Transform and precision findings

The actual renderer uploads `CameraView::relative_transform` records: it subtracts camera from the object translation on the CPU before the GPU adds tiny local cells. The original layer convention is `origin + camera*(1-factor)`, followed by that one camera subtraction. At factor 0.22 the resulting displacement is `origin - camera*0.22`. Scar uses exactly this convention and composes child affine transforms under the parent. Reflected and nonuniform parents remain affine; nested rotated child scaling is not approximated as TRS.

Baseline proof demonstrates why this matters. Under a small rotated/nonuniform child, adjacent cached local cells have world spacing `(0.27474213, 0.039819717)`. Relative-first projection preserves it even at ±999,000. Directly forming both full world points before subtraction quantizes the difference to `(0.25, 0)`. This result concerns the operation order, not a new precision guarantee for every existing absolute `push_pixel` cosmetic.

The independent Scar proof extracted **162 actual packets**: camera zero and both ±999,000.3/999,000.2 combinations; zoom 0.6, 1, 1.6; full/reduced; three rotations; and scales `(-0.9,0.3)`, `(0.3,-0.9)` and uniform 0.68. Every occupied quad corner was finite and remained within the conservative `80*max(abs(scale)) + 0.25` world-unit envelope. Actual surface-cell spacing through `CameraView` matched the real parent basis, including the reduced body's doubled local scale. Maximum measured spacing discrepancy was **0.0000030141628 logical units**. Whole-surface translation differed from the culling-relative origin by at most **0.0625 world units**, reflecting f32 translation quantization. Local detail is preserved within the group; absolute far-world placement is not infinitely precise.

Offscreen tests used the actual world extent at each zoom and both near/far cameras. Complete groups beyond each extent plus the conservative radius emitted no batches and spent no budget. Partially visible motifs deliberately remain complete groups; this is bounded scenery extraction, not per-cell viewport trimming or a full-warning gameplay admission rule.

## Batch, cache and budget findings

The independent extraction started with 127 existing object transforms, forcing the first Scar part to fill the 128-slot portable batch and its remaining parts to enter the next batch. Both qualities retained valid local transform indices, materials `[0,0]`, exact occupied counts and order. Four repeated pooled-frame rebuilds per quality remained correct. A subsequent insufficient-budget call preserved every preexisting pixel and transform as well as the supplied remainder.

| Quality | Occupied quads | Object transforms | Region budget after maximum gas | Remainder after motif |
|---|---:|---:|---:|---:|
| Full | 4,854 | 4 | 5,200 | 346 |
| Reduced | 1,411 | 3 | 2,600 | 1,189 |

All runtime body construction is inside one `LazyLock`; extraction borrows the cached bodies. The low surface contains real occupied 48×48 cells scaled twice locally and omits the back plume. It does not pretend alpha-zero cells save instances. The extractor counts the selected complete motif before appending it, atomically skips if it cannot fit, and subtracts only the actual motif count from the caller's remainder. There is no per-object independent budget or orphan plume/fragment path.

The **6,000/3,000 cap belongs to this regions call**, not the whole Game's backgrounds, world particles or UI. Existing gas uses up to 800/400 first, and the earlier layer-0 nebula/supernova can consume additional budget before layer 1. Thus fitting after maximum gas alone does not guarantee a naturally selected Scar will be drawn. Whole-group rejection under that existing priority is coherent. A guaranteed showcase needs an explicit priority/reservation decision; increasing the cap or adding a second budget would defeat this contract.

## Smallest production integration

Keep the author's API unchanged and copy its single module plus atlas/source/provenance to production-owned locations; adjust the one relative `include_str!` path. At the existing layer-1 factor-0.22 planet branch, replace the old planet/rings for immutable biome kind 4. Preserve the seed hash, `h.is_multiple_of(3)` admission, jittered unencoded `origin` and reduced mode's stable half-field selection. Do not append both complete objects.

Pass the interpolated presentation camera, already zoom-divided world viewport, immutable seeded rotation and proposed uniform 0.64–0.72 scale. Pass **unencoded origin**, not `position` which already includes parallax. Lend `scene.frame` and the same `scene.remaining` directly; do not decrement the budget again. Forward reduced quality and either the actual Reduced Motion flag or existing frozen cosmetic time. No physics, radiation hazard, actor, save, audio or RNG hook is needed.

Durable integration gates should check the actual region branch produces the authoritative cells when sufficient remainder exists; branch rejection leaves existing packets intact; the total regions append delta remains ≤6,000/3,000; reduced mode preserves deterministic field selection and uses the low object; Reduced Motion freezes transforms/colors; and both normal renderer builds accept the relative include path without work-only imports. These are future production gates, not already executed integrated-Game evidence.

## Visual scope

The pinned extracted 12-frame preview was visually inspected. Broad purple crater masses, branching lime faults, the missing limb, fragment and restrained wisps distinguish the motif at the three illustrated zooms. Reduced mode makes the seams chunkier. Fine individual full-quality cells become subpixel at zoom 0.6 and need not remain individually readable. The preview is software sRGB polygon blending from actual packets; browser vertex coverage, linear blending, background contrast and integer upscale can differ.

Next visual acceptance should use naturally seeded Radial Wastes fields in the real WebGPU and WebGL2 builds at zoom 0.6/1/1.6, both quality settings and Reduced Motion, including viewport-edge passage and far-world travel. Judge the broad silhouette among existing stars/gas and combat effects, verify no brightness/flicker issue, and profile actual frame extraction/rendering. Native test duration and occupied-cell counts are not GPU or phone frame-rate measurements.
