# Pixel asset brief and review record

Copy this brief for each new asset under `artifacts/concepts/` or the asset's source directory. Fill it with actual decisions and observations. A generated image is a reference until its cells and runtime behavior pass review.

## Brief

- Asset ID, role and intended biome/faction:
- Accepted style references and palette source:
- Silhouette and material hierarchy: readable outer shape, major planes, then small surface details.
- Native cell dimensions, cell size and collision envelope:
- Parent group, pivot/anchor intent, layer, parallax and affine placement:
- Occupancy policy: solid body or explicitly intentional holes/detached parts:
- Maximum occupied cells, transforms, animation frames and particle budget:
- Animation intent, frame timing and Reduced Motion behavior:
- Gameplay interaction: damage holes, joints, surviving emitters, collision and lights:
- Required views: normal scale, zoom limits, dark/light biome contrast and both renderer backends:

## Generation and authoring

Record the actual generator/workflow, exposed model/version, prompt, reference inputs, output hashes and available license/provenance notes. Leave unknown metadata unknown. Keep generated references separate from accepted authored native cells.

Author within the native grid and accepted palette. If converting a reference directly, specify the crop and every lossy policy in the manifest. Inspect and record each measured loss. Never silently remove detached parts, close holes, resize, dither or infer a pivot. Do not bake background gas, glow or parallax into a collision body; use explicit relative objects/materials where gameplay requires them.

## Evidence before integration

1. Pack and validate with the pinned offline tooling in [ASSET_PIPELINE.md](ASSET_PIPELINE.md). Record source/output hashes, occupancy, topology, palette and loss metrics.
2. Extract the candidate through the real `PixelBody`/`FrameBuilder` adapter and audit every color and all four transformed cell corners. Inspect nested transforms and animation boundaries. Metadata alone does not install a live Game animation or anchor.
3. Review the occupied-cell preview against the brief. Record the actual author and independent reviewer contributions, findings, repairs and exact commits in Mission Control. A software preview cannot certify GPU lighting or human readability.
4. Explicitly integrate the accepted asset into its runtime consumer. Verify destruction/collision, bounded groups, animation/motion settings, both GPU backends and the required scale/contrast views. If an engine change is needed, give it a separate bounded contract and regression evidence.
5. Keep unmeasured human polish, hearing, natural gameplay and physical-device gates pending. Link existing artifacts; do not invent reviews or screenshots.

## Acceptance record

- Candidate commit and exact source/output hashes:
- Author and reviewer:
- Tool/native/GPU results and artifact links:
- Findings and repair commit:
- Accepted runtime consumer and metadata boundary:
- Remaining verification or follow-up task IDs:
