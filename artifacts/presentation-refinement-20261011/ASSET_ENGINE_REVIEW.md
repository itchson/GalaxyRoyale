# Independent pixel asset pipeline review

Reviewed commit: `8f3d81848783c907cb520f3c35883658ae2fcbbb`, in the isolated `codex/pixel-art-refinement` checkout. No blocking defect found for integrating this offline authoring/validation tooling. The reviewed commit adds no shipping engine ABI or runtime asset change.

## Actual independent evidence

`python -B work/asset-pipeline-engine-review/review.py` passed. The owned harness pinned 33 working-tree inputs and their committed blobs, ran the actual 26 Python tests, validated the example, and re-audited the preserved historical native packets. Python was 3.12.10; Pillow was the pinned 12.3.0. The reviewed checkout was clean before and after, and all pinned bytes were unchanged.

All 24 hashes declared in the author's validation record match the current raw files. Twelve of our 33 inputs match Git blobs byte-for-byte; 21 match after CRLF-to-LF normalization. Both forms are recorded rather than presented as identical raw bytes.

The Python tests exercise actual game/UI/boss atlas palette/index/dimension round trips, duplicate/unused palette identity, alpha/grid/crop policies, topology, repeated-instance budgets, nested shear/reflection, timeline references, bounds and malformed inputs. Logs are retained in `python-tests.log`.

The accepted dock in the example is exactly the source atlas's palette and indices: 1,454 occupied cells. Each group adds two 32-cell lamp instances, totaling 1,518 cells/3 transforms/1 batch. Historical packet re-audit compares every color and all four corners; maximum corner error is `3.4393310528457732e-6` world units, and maximum occupied-bound error is `9.918212882098487e-7`. I also inspected the retained software preview: the open landing bay and detached lamp cells remain visible. This is offline packet/raster evidence, not GPU presentation.

## Source contract review

Exact conversion is the default. Uniform cell-grid decoding rejects mixed blocks; crop is explicit; alpha threshold, component removal, nearest-fit resizing and quantization require declared policies with loss counters. Indexed PNG round trips preserve duplicate palette indices; RGBA input intentionally cannot infer that identity. Transparent hidden RGB is canonicalized and recorded.

Four-neighbor topology distinguishes enclosed holes from all edge-connected exterior regions. No implicit hole filling or component removal is introduced. Opaque legacy extension fields remain verbatim, without a false claim that this generic tool validates their gameplay meanings.

Groups charge every occupied sprite instance, including reused sprites. Full parent affine composition matches the actual engine basis convention; bounds include four corners of each occupied cell. Native group preflight completes before any FrameBuilder append. The actual default unlit material and 16-byte instance/32-byte transform contract remain unchanged. The source includes the meaningful 129-transform batch rollover test; this review does not claim its final execution.

## Explicit boundaries

The final native five-test executable was blocked by Windows Application Control OS4551. I did not compile, execute, rename, copy, or attempt to bypass that executable. The author's strict Clippy result is compile-only; final native execution remains Pending on a clean runner. The retained example predates final utility guards and lacks a retained original executable fingerprint/sample times. Its historical classification is accurate.

The `0.0001` audit tolerance is a reject threshold, not a precision guarantee throughout the allowed ±1,000,000 authoring extent. Actual native transforms are f32. Generic metadata also permits finite singular affines; a future collision adapter must decide whether to admit them. Those are future consumer constraints, not defects in this offline render proof.

The offline native utility reconstructs bodies per part/frame. Its occupied-cell caps bound submission, not all work involved in scanning a sparsely occupied source grid. A future repeated runtime path should cache bodies by sprite and budget conversion separately. No native timing or phone claim is made here.

Animation metadata does not automatically drive Game occupancy, damage, collisions, anchors, Reduced Motion or abilities. Both backend presentation, human moving readability, physical-phone acceptance and any runtime consumer remain Pending. No browser, server, production source, dependency or dashboard mutation was performed by this review.

## Rechecking this freeze

`python -B work/asset-pipeline-engine-review/verify.py --live` verifies the frozen files, exact reviewed commit blobs, and current pinned source bytes without rerunning tests or producing output files. `frozen-hashes.json` records the immutable evidence. This report does not supersede historical Wiki, audio, engine or browser reviews.
