# Pixel normals and luminance masks

The first opt-in surface material uses the existing pixel quad stream. A cell carries an octahedrally encoded normal and a linear luminance multiplier in the unused high bits of its transform word. Pixel instances remain 16 bytes, affine transforms remain 32 bytes, and each batch retains 128 transform slots. Unmapped cells retain their previous lighting and emission behavior.

## Coordinates and shading

Local +X is right, +Y is down, and +Z faces the viewer. Normals use the inverse transpose of the composed object/group affine, embedded with Z scale `sqrt(abs(det A))`. Uniform scaling therefore preserves the surface slope. Rotation, reflection, nonuniform scale and hierarchy shear are handled before camera projection; zoom and the screen Y flip do not alter a material normal. The GPU uses f32 and falls back to +Z for a zero or near-singular normalized basis. The portable reference calculation uses f64.

The shared WGSL shader decodes these values and applies Lambert response to the existing eight bounded point lights. Light height occupies a previously unused uniform component. No normal texture, extra draw pass, per-pixel light, shadow pass or new runtime dependency is introduced. The existing soft-particle coverage filter remains separate from hard-cell geometry.

The luminance byte multiplies an object's existing emission: 0 disables self-emission, 255 retains it. It is not an automatic light source or independent brightness. Albedo remains the authored cell color. The first PAN study gives existing signal lamps emission while keeping hull plates and glass non-emissive.

## First authored material

The PAN starter study has 662 coarse hull cells and 1,205 fine detail cells. Semantic plate, recess and glass heights produce normals independently of painted brightness. Its 31 existing fine signal cells retain emission; all other fine cells and all coarse cells have mask zero. The imported map is immutable rendering data and does not change collision occupancy, damage, rewards or progression.

The offline authoring and validation tools are documented in [PAN_SURFACE_AUTHORING.md](PAN_SURFACE_AUTHORING.md). Convert the reviewed semantic record using the real portable Rust encoder:

```powershell
cargo run --locked -p engine_core --example pack_raicho_surfaces -- artifacts/pixel-surface-20261012/pan-material-proposal.json assets/pixels/pan-surface/material.json
```

The output is bound to the exact source atlas by SHA-256 provenance and a runtime FNV64 check. Cell index order, shape, valid-bit encoding, signal masks and counts are validated. Coarse occupancy owns the fine overlay: removed hull cells also suppress their corresponding fine quads, without shifting the material indices of surviving cells.

## Verification boundary

Portable encoding, affine reference fixtures, shader validation, packing correspondence, native gameplay extraction and actual browser lighting are separate gates. Consult [the current evidence record](../artifacts/pixel-surface-20261012/STATUS.md) for completed checks and remaining acceptance. The material rollout is limited to the registered PAN starter; broader hull, asteroid, dock and background authoring remains future work. Physical-device performance and extended soak testing are pending.

The user's inspiration is [Moonfell's public normal/luminance demonstration](https://x.com/moonfell_rpg/status/2109451515157233916?s=20). We inspected the public demonstration; no external tool code or artwork was imported.
